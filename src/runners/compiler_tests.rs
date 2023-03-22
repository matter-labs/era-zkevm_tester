use super::*;

use crate::default_environment::*;
use crate::runners::events::SolidityLikeEvent;
use crate::runners::hashmap_based_memory::SimpleHashmapMemory;
use crate::runners::simple_witness_tracer::MemoryLogWitnessTracer;
use crate::utils::IntoFixedLengthByteIterator;
use crate::{Address, H256, U256};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::atomic::AtomicU64;
use zk_evm::block_properties::*;
use zk_evm::precompiles::DefaultPrecompilesProcessor;
use zk_evm::reference_impls::decommitter::SimpleDecommitter;
use zk_evm::reference_impls::event_sink::{EventMessage, InMemoryEventSink};
use zk_evm::testing::storage::InMemoryStorage;
use zk_evm::vm_state::*;
use zk_evm::zkevm_opcode_defs::decoding::AllowedPcOrImm;
use zk_evm::zkevm_opcode_defs::decoding::{
    EncodingModeProduction, EncodingModeTesting, VmEncodingMode,
};
use zk_evm::zkevm_opcode_defs::definitions::ret::RET_IMPLICIT_RETURNDATA_PARAMS_REGISTER;
use zk_evm::zkevm_opcode_defs::system_params::DEPLOYER_SYSTEM_CONTRACT_ADDRESS;
use zk_evm::zkevm_opcode_defs::FatPointer;
use zk_evm::{aux_structures::*, GenericNoopTracer};
use zkevm_assembly::Assembly;

use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash)]
pub struct FullABIParams {
    pub is_constructor: bool,
    pub is_system_call: bool,
    pub r3_value: Option<U256>,
    pub r4_value: Option<U256>,
    pub r5_value: Option<U256>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash)]
pub enum VmLaunchOption {
    Default,
    Pc(u32),
    Label(String),
    Call,
    Constructor,
    ManualCallABI(FullABIParams),
}

#[derive(Debug)]
pub enum VmExecutionResult {
    Ok(Vec<u8>),
    Revert(Vec<u8>),
    Panic,
    MostLikelyDidNotFinish(Address, u64),
}

#[derive(Debug, Default)]
pub struct VmExecutionContext {
    pub this_address: Address,
    pub msg_sender: Address,
    pub u128_value: u128,
    pub transaction_index: u32,
}

impl VmExecutionContext {
    pub fn new(
        this_address: Address,
        msg_sender: Address,
        u128_value: u128,
        transaction_index: u32,
    ) -> Self {
        Self {
            this_address,
            msg_sender,
            u128_value,
            transaction_index,
        }
    }
}

#[derive(Debug)]
pub struct MemoryArea {
    pub words: Vec<U256>,
}

impl MemoryArea {
    pub fn empty() -> Self {
        Self { words: vec![] }
    }

    pub fn dump_be_bytes(&self, range: std::ops::Range<usize>) -> Vec<u8> {
        if range.is_empty() {
            return vec![];
        }

        let mut result = Vec::with_capacity(range.len());

        let starting_word = range.start % 32;
        let start_bytes = range.start / 32;
        if start_bytes != 0 {
            let el = self
                .words
                .get(starting_word)
                .copied()
                .unwrap_or(U256::zero());
            let mut buffer = [0u8; 32];
            el.to_big_endian(&mut buffer);
            result.extend_from_slice(&buffer[(32 - start_bytes)..]);
        }

        let end_cap = range.end % 32;
        let end_word = range.end / 32;

        // now just iterate aligned
        let range_start = if start_bytes == 0 {
            starting_word
        } else {
            starting_word + 1
        };

        let range_end = if end_cap == 0 {
            end_word
        } else {
            if end_word == 0 {
                end_word
            } else {
                end_word - 1
            }
        };

        for i in range_start..range_end {
            let el = self.words.get(i).copied().unwrap_or(U256::zero());
            let mut buffer = [0u8; 32];
            el.to_big_endian(&mut buffer);
            result.extend_from_slice(&buffer[..]);
        }

        if end_cap != 0 {
            let el = self.words.get(end_word).copied().unwrap_or(U256::zero());
            let mut buffer = [0u8; 32];
            el.to_big_endian(&mut buffer);
            result.extend_from_slice(&buffer[..end_cap]);
        }

        result
    }
}

pub fn calldata_to_aligned_data(calldata: &Vec<u8>) -> Vec<U256> {
    if calldata.len() == 0 {
        return vec![];
    }
    let mut capacity = calldata.len() / 32;
    if calldata.len() % 32 != 0 {
        capacity += 1;
    }
    let mut result = Vec::with_capacity(capacity);
    let mut it = calldata.chunks_exact(32);
    for el in &mut it {
        let el = U256::from_big_endian(el);
        result.push(el);
    }
    let mut buffer = [0u8; 32];
    buffer[0..it.remainder().len()].copy_from_slice(it.remainder());
    let el = U256::from_big_endian(&buffer);
    result.push(el);

    result
}

pub(crate) fn dump_memory_page_using_primitive_value(
    memory: &SimpleHashmapMemory,
    ptr: PrimitiveValue,
) -> Vec<u8> {
    if ptr.is_pointer == false {
        return vec![];
    }
    let fat_ptr = FatPointer::from_u256(ptr.value);
    dump_memory_page_using_fat_pointer(memory, fat_ptr)
}

pub(crate) fn dump_memory_page_using_fat_pointer(
    memory: &SimpleHashmapMemory,
    fat_ptr: FatPointer,
) -> Vec<u8> {
    dump_memory_page_by_offset_and_length(
        memory,
        fat_ptr.memory_page,
        (fat_ptr.start + fat_ptr.offset) as usize,
        (fat_ptr.length - fat_ptr.offset) as usize,
    )
}

pub(crate) fn fat_ptr_into_page_and_aligned_words_range(
    ptr: PrimitiveValue,
) -> (u32, std::ops::Range<u32>) {
    if ptr.is_pointer == false {
        return (0, 0..0);
    }
    let fat_ptr = FatPointer::from_u256(ptr.value);
    let beginning_word = (fat_ptr.start + fat_ptr.offset) / 32;
    let end = fat_ptr.start + fat_ptr.length;
    let mut end_word = end / 32;
    if end % 32 != 0 {
        end_word += 1;
    }

    (fat_ptr.memory_page, beginning_word..end_word)
}

pub(crate) fn dump_memory_page_by_offset_and_length(
    memory: &SimpleHashmapMemory,
    page: u32,
    offset: usize,
    length: usize,
) -> Vec<u8> {
    assert!(offset < (1u32 << 24) as usize);
    assert!(length < (1u32 << 24) as usize);
    let mut dump = Vec::with_capacity(length);
    if length == 0 {
        return dump;
    }

    let first_word = offset / 32;
    let end_byte = offset + length;
    let mut last_word = end_byte / 32;
    if end_byte % 32 != 0 {
        last_word += 1;
    }

    let unalignment = offset % 32;

    let page_part =
        memory.dump_page_content_as_u256_words(page, (first_word as u32)..(last_word as u32));

    let mut is_first = true;
    let mut remaining = length;
    for word in page_part.into_iter() {
        let mut it = word.into_be_iter();
        if is_first {
            is_first = false;
            let mut it = it.skip(unalignment);
            while let Some(next) = it.next() {
                if remaining > 0 {
                    dump.push(next);
                    remaining -= 1;
                }
            }
        } else {
            while let Some(next) = it.next() {
                if remaining > 0 {
                    dump.push(next);
                    remaining -= 1;
                }
            }
        }
    }

    assert_eq!(
        dump.len(),
        length,
        "tried to dump with offset {}, length {}, got a bytestring of length {}",
        offset,
        length,
        dump.len()
    );

    dump
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct StorageKey {
    pub address: Address,
    pub key: U256,
}

impl StorageKey {
    pub fn into_raw_key(self) -> U256 {
        let mut hasher = Sha256::new();
        hasher.update(self.address.as_bytes());
        let mut buffer = [0u8; 32];
        self.key.to_big_endian(&mut buffer);
        hasher.update(&buffer);

        let result = hasher.finalize();

        let key = U256::from_big_endian(&result.as_slice());

        key
    }

    fn format_as_hex(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StorageKey")
            .field("address", &format!("{:?}", &self.address))
            .field("key", &format!("0x{:x}", &self.key))
            .finish()
    }
}

impl std::fmt::Display for StorageKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format_as_hex(f)
    }
}

impl std::fmt::Debug for StorageKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format_as_hex(f)
    }
}

#[derive(Debug)]
pub struct VmSnapshot {
    pub registers: [PrimitiveValue; zk_evm::zkevm_opcode_defs::REGISTERS_COUNT],
    pub flags: zk_evm::flags::Flags,
    pub timestamp: u32,
    pub memory_page_counter: u32,
    pub tx_number_in_block: u16,
    pub previous_super_pc: u32,
    pub did_call_or_ret_recently: bool,
    pub calldata_area_dump: MemoryArea,
    pub returndata_area_dump: MemoryArea,
    pub execution_has_ended: bool,
    pub stack_dump: MemoryArea,
    pub heap_dump: MemoryArea,
    pub storage: HashMap<StorageKey, H256>,
    pub deployed_contracts: HashMap<Address, Assembly>,
    pub execution_result: VmExecutionResult,
    pub returndata_bytes: Vec<u8>,
    pub raw_events: Vec<EventMessage>,
    pub to_l1_messages: Vec<EventMessage>,
    pub events: Vec<SolidityLikeEvent>,
    pub serialized_events: String,
    pub num_cycles_used: usize,
}

#[derive(Debug)]
pub struct RawInMemoryStorage {
    pub values: HashMap<StorageKey, H256>,
    pub contracts: HashMap<Address, Assembly>,
    pub factory_deps: HashMap<H256, Vec<u8>>,
}

pub fn default_entry_point_contract_address() -> Address {
    Address::from_low_u64_be(1234567u64)
}

///
/// Used for testing the compiler with a single contract.
///
#[allow(clippy::too_many_arguments)]
pub async fn run_vm(
    test_name: String,
    assembly: Assembly,
    calldata: Vec<u8>,
    storage: HashMap<StorageKey, H256>,
    context: Option<VmExecutionContext>,
    vm_launch_option: VmLaunchOption,
    cycles_limit: usize,
    known_contracts: HashMap<U256, Assembly>,
    default_aa_code_hash: U256,
) -> anyhow::Result<VmSnapshot> {
    let entry_address = default_entry_point_contract_address();
    let mut contracts: HashMap<Address, Assembly> = HashMap::new();
    contracts.insert(entry_address, assembly);
    run_vm_multi_contracts(
        test_name,
        contracts,
        calldata,
        storage,
        entry_address,
        context,
        vm_launch_option,
        cycles_limit,
        known_contracts,
        default_aa_code_hash,
    )
    .await
}

pub struct ExtendedTestingTools<const B: bool> {
    pub storage: InMemoryStorage,
    pub memory: SimpleHashmapMemory,
    pub event_sink: InMemoryEventSink,
    pub precompiles_processor: DefaultPrecompilesProcessor<B>,
    pub decommittment_processor: SimpleDecommitter<B>,
    pub witness_tracer: MemoryLogWitnessTracer,
}

pub fn create_default_testing_tools() -> ExtendedTestingTools<false> {
    let storage = InMemoryStorage::new();
    let memory = SimpleHashmapMemory::new();
    let event_sink = InMemoryEventSink::new();
    let precompiles_processor = DefaultPrecompilesProcessor::<false>;
    let decommittment_processor = SimpleDecommitter::<false>::new();
    let witness_tracer = MemoryLogWitnessTracer {
        is_dummy: false,
        queries: vec![],
    };

    ExtendedTestingTools::<false> {
        storage,
        memory,
        event_sink,
        precompiles_processor,
        decommittment_processor,
        witness_tracer,
    }
}

pub fn create_vm<'a, const B: bool, const N: usize, E: VmEncodingMode<N>>(
    tools: &'a mut ExtendedTestingTools<B>,
    block_properties: &'a BlockProperties,
    context: VmExecutionContext,
    contracts: &HashMap<Address, Assembly>,
    known_contracts: HashMap<U256, Assembly>,
    initial_pc: E::PcOrImm,
) -> (
    VmState<
        'a,
        InMemoryStorage,
        SimpleHashmapMemory,
        InMemoryEventSink,
        DefaultPrecompilesProcessor<B>,
        SimpleDecommitter<B>,
        MemoryLogWitnessTracer,
        N,
        E,
    >,
    HashMap<U256, Assembly>,
) {
    use zk_evm::contract_bytecode_to_words;
    use zk_evm::utils::bytecode_to_code_hash_for_mode;
    // fill the decommitter

    let mut factory_deps: HashMap<U256, Vec<U256>> = HashMap::new();
    let mut reverse_lookup_for_assembly = HashMap::new();

    for (address, assembly) in contracts.iter() {
        let bytecode = assembly
            .clone()
            .compile_to_bytecode_for_mode::<N, E>()
            .expect("must compile an assembly");
        let bytecode_hash = bytecode_to_code_hash_for_mode::<N, E>(&bytecode).unwrap();
        let address_as_u256 = U256::from_big_endian(&address.as_bytes());
        let bytecode_hash_as_u256 = U256::from_big_endian(&bytecode_hash);

        reverse_lookup_for_assembly.insert(bytecode_hash_as_u256, assembly.clone());

        // add to decommitter
        let bytecode_words = contract_bytecode_to_words(&bytecode);
        let _existing = factory_deps.insert(bytecode_hash_as_u256, bytecode_words);
    }

    for (bytecode_hash, assembly) in known_contracts.into_iter() {
        let mut assembly = assembly;
        let bytecode = assembly
            .compile_to_bytecode_for_mode::<N, E>()
            .expect("must compile an assembly");
        let bytecode_words = contract_bytecode_to_words(&bytecode);
        let _ = factory_deps.insert(bytecode_hash, bytecode_words);
        reverse_lookup_for_assembly.insert(bytecode_hash, assembly);
    }

    let decommitter_els: Vec<_> = factory_deps.into_iter().into_iter().collect();

    tools.decommittment_processor.populate(decommitter_els);

    let mut vm = VmState::empty_state(
        &mut tools.storage,
        &mut tools.memory,
        &mut tools.event_sink,
        &mut tools.precompiles_processor,
        &mut tools.decommittment_processor,
        &mut tools.witness_tracer,
        block_properties,
    );

    let initial_context = CallStackEntry {
        this_address: context.this_address,
        msg_sender: context.msg_sender,
        code_address: context.this_address,
        base_memory_page: MemoryPage(INITIAL_BASE_PAGE),
        code_page: MemoryPage(ENTRY_POINT_PAGE),
        sp: E::PcOrImm::from_u64_clipped(0),
        pc: initial_pc,
        exception_handler_location: E::PcOrImm::max(),
        ergs_remaining: zk_evm::zkevm_opcode_defs::system_params::VM_INITIAL_FRAME_ERGS,
        this_shard_id: 0,
        caller_shard_id: 0,
        code_shard_id: 0,
        is_static: false,
        is_local_frame: false,
        context_u128_value: context.u128_value,
        heap_bound: 0,
        aux_heap_bound: 0,
    };

    // we consider the tested code as a bootloader
    vm.push_bootloader_context(0, initial_context);
    vm.local_state.timestamp = INITIAL_TIMESTAMP;
    vm.local_state.memory_page_counter = INITIAL_MEMORY_COUNTER;
    vm.local_state.tx_number_in_block = context.transaction_index as u16;

    (vm, reverse_lookup_for_assembly)
}

pub(crate) fn vm_may_have_ended<'a, const B: bool, const N: usize, E: VmEncodingMode<N>>(
    vm: &VmState<
        'a,
        InMemoryStorage,
        SimpleHashmapMemory,
        InMemoryEventSink,
        DefaultPrecompilesProcessor<B>,
        SimpleDecommitter<B>,
        MemoryLogWitnessTracer,
        N,
        E,
    >,
) -> Option<VmExecutionResult> {
    let execution_has_ended = vm.execution_has_ended();

    let r1 = vm.local_state.registers[RET_IMPLICIT_RETURNDATA_PARAMS_REGISTER as usize];
    let current_address = vm.local_state.callstack.get_current_stack().this_address;

    let outer_eh_location = E::PcOrImm::max().as_u64();
    match (
        execution_has_ended,
        vm.local_state.callstack.get_current_stack().pc.as_u64(),
    ) {
        (true, 0) => {
            let returndata = dump_memory_page_using_primitive_value(&vm.memory, r1);

            Some(VmExecutionResult::Ok(returndata))
        }
        (false, _) => None,
        (true, l) if l == outer_eh_location => {
            // check r1,r2,r3
            if vm.local_state.flags.overflow_or_less_than_flag {
                Some(VmExecutionResult::Panic)
            } else {
                let returndata = dump_memory_page_using_primitive_value(&vm.memory, r1);
                Some(VmExecutionResult::Revert(returndata))
            }
        }
        (_, a) => Some(VmExecutionResult::MostLikelyDidNotFinish(
            current_address,
            a,
        )),
    }
}

///
/// Used for testing the compiler with multiple contracts.
///
#[allow(clippy::too_many_arguments)]
pub async fn run_vm_multi_contracts(
    test_name: String,
    contracts: HashMap<Address, Assembly>,
    calldata: Vec<u8>,
    storage: HashMap<StorageKey, H256>,
    entry_address: Address,
    context: Option<VmExecutionContext>,
    vm_launch_option: VmLaunchOption,
    cycles_limit: usize,
    known_contracts: HashMap<U256, Assembly>,
    default_aa_code_hash: U256,
) -> anyhow::Result<VmSnapshot> {
    use zkevm_assembly::{get_encoding_mode, RunningVmEncodingMode};
    let encoding_mode = get_encoding_mode();
    match encoding_mode {
        RunningVmEncodingMode::Production => {
            run_vm_multi_contracts_inner::<8, EncodingModeProduction>(
                test_name,
                contracts,
                calldata,
                storage,
                entry_address,
                context,
                vm_launch_option,
                cycles_limit,
                known_contracts,
                default_aa_code_hash,
            )
            .await
        }
        RunningVmEncodingMode::Testing => {
            run_vm_multi_contracts_inner::<16, EncodingModeTesting>(
                test_name,
                contracts,
                calldata,
                storage,
                entry_address,
                context,
                vm_launch_option,
                cycles_limit,
                known_contracts,
                default_aa_code_hash,
            )
            .await
        }
    }
}

///
/// Used for testing the compiler with multiple contracts.
///
#[allow(clippy::too_many_arguments)]
async fn run_vm_multi_contracts_inner<const N: usize, E: VmEncodingMode<N>>(
    test_name: String,
    contracts: HashMap<Address, Assembly>,
    calldata: Vec<u8>,
    storage: HashMap<StorageKey, H256>,
    entry_address: Address,
    context: Option<VmExecutionContext>,
    vm_launch_option: VmLaunchOption,
    cycles_limit: usize,
    known_contracts: HashMap<U256, Assembly>,
    default_aa_code_hash: U256,
) -> anyhow::Result<VmSnapshot> {
    let mut contracts = contracts;
    for (a, c) in contracts.iter_mut() {
        match c.compile_to_bytecode_for_mode::<N, E>() {
            Ok(_) => {}
            Err(e) => {
                panic!(
                    "failed to compile bytecode at address {:?} due to error {}",
                    a, e
                );
            }
        }
    }
    let all_contracts_mapping = contracts.clone();

    let (initial_pc, set_far_call_props, extra_props) = match &vm_launch_option {
        VmLaunchOption::Pc(pc) => (E::PcOrImm::from_u64_clipped(*pc as u64), false, None),
        VmLaunchOption::Label(label) => {
            let offset = *contracts
                .get(&entry_address)
                .unwrap()
                .function_labels
                .get(label)
                .unwrap();

            (E::PcOrImm::from_u64_clipped(offset as u64), false, None)
        }
        VmLaunchOption::Default | VmLaunchOption::Call => {
            (E::PcOrImm::from_u64_clipped(0u64), true, None)
        }

        VmLaunchOption::Constructor => (
            E::PcOrImm::from_u64_clipped(0u64),
            true,
            Some(FullABIParams {
                is_constructor: true,
                is_system_call: false,
                r3_value: None,
                r4_value: None,
                r5_value: None,
            }),
        ),
        VmLaunchOption::ManualCallABI(value) => (
            E::PcOrImm::from_u64_clipped(0u64),
            true,
            Some(value.clone()),
        ),
    };

    let mut tools = create_default_testing_tools();
    let mut block_properties = create_default_block_properties();
    block_properties.default_aa_code_hash = default_aa_code_hash;

    let calldata_length = calldata.len();
    use zk_evm::contract_bytecode_to_words;

    // fill the calldata
    let aligned_calldata = calldata_to_aligned_data(&calldata);
    // and initial memory page
    let initial_assembly = contracts
        .get(&entry_address)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Initial assembly not found"))?;
    let initial_bytecode = initial_assembly
        .clone()
        .compile_to_bytecode_for_mode::<N, E>()
        .unwrap();
    let initial_bytecode_as_memory = contract_bytecode_to_words(&initial_bytecode);

    tools.memory.populate(vec![
        (CALLDATA_PAGE, aligned_calldata),
        (ENTRY_POINT_PAGE, initial_bytecode_as_memory),
    ]);

    // fill the storage. Only rollup shard for now
    for (key, value) in storage.into_iter() {
        let per_address_entry = tools.storage.inner[0].entry(key.address).or_default();
        per_address_entry.insert(key.key, U256::from_big_endian(&value.as_bytes()));
    }

    // some context notion
    let context = context.unwrap_or_else(|| {
        let mut ctx = VmExecutionContext::default();
        ctx.this_address = entry_address;

        ctx
    });

    // fill the rest
    let (mut vm, reverse_lookup_for_assembly) = create_vm::<false, N, E>(
        &mut tools,
        &block_properties,
        context,
        &contracts,
        known_contracts,
        initial_pc,
    );

    if set_far_call_props {
        // we need to properly set calldata abi
        vm.local_state.registers[0] =
            crate::utils::form_initial_calldata_ptr(CALLDATA_PAGE, calldata_length as u32);

        vm.local_state.registers[1] = PrimitiveValue::empty();
        vm.local_state.registers[2] = PrimitiveValue::empty();
        vm.local_state.registers[3] = PrimitiveValue::empty();

        if let Some(extra_props) = extra_props {
            let mut r2_value = U256::zero();
            if extra_props.is_constructor {
                r2_value += U256::from(1u64 << 0);
            }
            if extra_props.is_system_call {
                r2_value += U256::from(1u64 << 1);
            }

            let r3_value = extra_props.r3_value.unwrap_or(U256::zero());
            let r4_value = extra_props.r4_value.unwrap_or(U256::zero());
            let r5_value = extra_props.r5_value.unwrap_or(U256::zero());

            vm.local_state.registers[1] = PrimitiveValue::from_value(r2_value);
            vm.local_state.registers[2] = PrimitiveValue::from_value(r3_value);
            vm.local_state.registers[3] = PrimitiveValue::from_value(r4_value);
            vm.local_state.registers[4] = PrimitiveValue::from_value(r5_value);
        }
    }

    let mut result = None;

    let mut cycles_used = 0;

    match get_tracing_mode() {
        VmTracingOptions::None => {
            vm.witness_tracer.is_dummy = true;
            let mut tracer = GenericNoopTracer::new();
            for _ in 0..cycles_limit {
                vm.cycle(&mut tracer);
                cycles_used += 1;

                // early return
                if let Some(end_result) = vm_may_have_ended(&vm) {
                    result = Some(end_result);
                    break;
                }
            }
        }
        VmTracingOptions::TraceDump => {
            use crate::trace::*;

            let mut tracer = VmDebugTracer::new_from_entry_point(entry_address, &initial_assembly);
            tracer.add_known_contracts(&all_contracts_mapping);

            for _ in 0..cycles_limit {
                vm.witness_tracer.queries.truncate(0);
                vm.cycle(&mut tracer);
                cycles_used += 1;

                // manually replace all memory interactions
                let last_step = tracer.steps.last_mut().unwrap();
                last_step.memory_interactions.truncate(0);
                for query in vm.witness_tracer.queries.drain(..) {
                    let memory_type = match query.location.memory_type {
                        zk_evm::abstractions::MemoryType::Heap => crate::trace::MemoryType::heap,
                        zk_evm::abstractions::MemoryType::AuxHeap => {
                            crate::trace::MemoryType::aux_heap
                        }
                        zk_evm::abstractions::MemoryType::FatPointer => {
                            crate::trace::MemoryType::fat_ptr
                        }
                        zk_evm::abstractions::MemoryType::Code => crate::trace::MemoryType::code,
                        zk_evm::abstractions::MemoryType::Stack => crate::trace::MemoryType::stack,
                    };

                    let page = query.location.page.0;
                    let address = query.location.index.0;
                    let value = format!("{:064x}", query.value);
                    let direction = if query.rw_flag {
                        crate::trace::MemoryAccessType::Write
                    } else {
                        crate::trace::MemoryAccessType::Read
                    };

                    let as_interaction = MemoryInteraction {
                        memory_type,
                        page,
                        address,
                        value,
                        direction,
                    };

                    last_step.memory_interactions.push(as_interaction);
                }

                // early return
                if let Some(end_result) = vm_may_have_ended(&vm) {
                    result = Some(end_result);
                    break;
                }
            }

            pub fn is_trace_enabled() -> bool {
                std::env::var("RUST_LOG")
                    .map(|variable| variable.contains("vm=trace"))
                    .unwrap_or_default()
            }

            if is_trace_enabled() {
                let VmDebugTracer {
                    steps,
                    debug_info: debug_infos_map,
                    ..
                } = tracer;

                let empty_callstack_dummy_debug_info = ContractSourceDebugInfo {
                    assembly_code: "nop r0, r0, r0, r0".to_owned(),
                    pc_line_mapping: HashMap::from([(0, 0)]),
                    active_lines: std::collections::HashSet::from([0]),
                };

                let mut sources = HashMap::new();

                sources.insert(
                    EMPTY_CONTEXT_HEX.to_owned(),
                    empty_callstack_dummy_debug_info,
                );

                for (address, info) in debug_infos_map.into_iter() {
                    sources.insert(format!("0x{}", hex::encode(address.as_bytes())), info);
                }

                let full_trace = VmTrace { steps, sources };
                output_execution_trace(full_trace, entry_address, test_name);
            }
        }
        VmTracingOptions::ManualVerbose => {
            vm.witness_tracer.is_dummy = true;
            use crate::runners::debug_tracer::DebugTracerWithAssembly;
            let mut tracer = DebugTracerWithAssembly {
                current_code_address: entry_address,
                code_address_to_assembly: all_contracts_mapping,
                _marker: std::marker::PhantomData,
            };
            for _ in 0..cycles_limit {
                vm.cycle(&mut tracer);
                cycles_used += 1;

                // early return
                if let Some(end_result) = vm_may_have_ended(&vm) {
                    result = Some(end_result);
                    break;
                }
            }
        }
    }

    let return_abi_register = vm.local_state.registers[0]; // r1

    let execution_result = if let Some(result) = result {
        result
    } else {
        let current_address = vm.local_state.callstack.get_current_stack().this_address;
        let pc = vm.local_state.callstack.get_current_stack().pc.as_u64();
        VmExecutionResult::MostLikelyDidNotFinish(current_address, pc)
    };

    let execution_has_ended = vm.execution_has_ended();

    let VmState {
        local_state,
        block_properties: _,
        ..
    } = vm;

    let mut result_storage = HashMap::new();
    let mut deployed_contracts = HashMap::new();

    let ExtendedTestingTools {
        storage,
        memory,
        event_sink,
        precompiles_processor: _,
        decommittment_processor: _,
        witness_tracer: _,
    } = tools;

    let (_full_history, raw_events, l1_messages) = event_sink.flatten();
    use crate::runners::events::merge_events;
    let events = merge_events(raw_events.clone());

    let (_history, _per_slot) = storage.clone().flatten_and_net_history();
    // dbg!(history);
    // dbg!(per_slot);

    let storage = storage.inner;
    let storage = storage.into_iter().next().unwrap();

    for (address, inner) in storage.into_iter() {
        for (key, value) in inner.into_iter() {
            let storage_key = StorageKey { address, key };
            let mut buffer = [0u8; 32];
            value.to_big_endian(&mut buffer);
            let value_h256 = H256::from_slice(&buffer);
            result_storage.insert(storage_key, value_h256);

            if address == *DEPLOYER_SYSTEM_CONTRACT_ADDRESS {
                let mut buffer = [0u8; 32];
                key.to_big_endian(&mut buffer);
                let deployed_address = Address::from_slice(&buffer[12..]);
                if let Some(known_assembly) = reverse_lookup_for_assembly.get(&value) {
                    deployed_contracts.insert(deployed_address, known_assembly.clone());
                }
            }
        }
    }

    // memory dump for returndata
    let returndata_page_content = if get_tracing_mode() != VmTracingOptions::None {
        if return_abi_register.is_pointer {
            let return_abi_ptr = FatPointer::from_u256(return_abi_register.value);
            memory.dump_full_page_as_u256_words(return_abi_ptr.memory_page)
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    let returndata_mem = MemoryArea {
        words: returndata_page_content,
    };

    let calldata_page_content = if get_tracing_mode() != VmTracingOptions::None {
        memory.dump_full_page_as_u256_words(CALLDATA_PAGE)
    } else {
        vec![]
    };

    let calldata_mem = MemoryArea {
        words: calldata_page_content,
    };

    let returndata_bytes = match &execution_result {
        VmExecutionResult::Ok(ref res) => res.clone(),
        VmExecutionResult::Revert(ref res) => res.clone(),
        VmExecutionResult::Panic => vec![],
        VmExecutionResult::MostLikelyDidNotFinish(..) => vec![],
    };

    let compiler_tests_events: Vec<crate::runners::events::Event> =
        events.iter().cloned().map(|el| el.into()).collect();

    let serialized_events = serde_json::to_string_pretty(&compiler_tests_events).unwrap();

    Ok(VmSnapshot {
        registers: local_state.registers,
        flags: local_state.flags,
        timestamp: local_state.timestamp,
        memory_page_counter: local_state.memory_page_counter,
        tx_number_in_block: local_state.tx_number_in_block,
        previous_super_pc: local_state.previous_super_pc.as_u64() as u32,
        did_call_or_ret_recently: local_state.did_call_or_ret_recently,
        calldata_area_dump: calldata_mem,
        returndata_area_dump: returndata_mem,
        execution_has_ended,
        stack_dump: MemoryArea::empty(),
        heap_dump: MemoryArea::empty(),
        storage: result_storage,
        deployed_contracts,
        execution_result,
        returndata_bytes,
        raw_events,
        to_l1_messages: l1_messages,
        events,
        serialized_events,
        num_cycles_used: cycles_used,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u64)]
pub enum VmTracingOptions {
    None = 0,
    TraceDump = 1,
    ManualVerbose = 2,
}

impl VmTracingOptions {
    pub const fn from_u64(value: u64) -> Self {
        match value {
            x if x == Self::None as u64 => Self::None,
            x if x == Self::TraceDump as u64 => Self::TraceDump,
            x if x == Self::ManualVerbose as u64 => Self::ManualVerbose,
            _ => unreachable!(),
        }
    }

    pub const fn as_u64(self) -> u64 {
        self as u64
    }
}

pub static TRACE_MODE: AtomicU64 = AtomicU64::new(VmTracingOptions::TraceDump as u64);

pub fn set_tracing_mode(value: VmTracingOptions) {
    TRACE_MODE.store(value.as_u64(), std::sync::atomic::Ordering::SeqCst);
}

pub fn get_tracing_mode() -> VmTracingOptions {
    VmTracingOptions::from_u64(TRACE_MODE.load(std::sync::atomic::Ordering::Relaxed))
}
