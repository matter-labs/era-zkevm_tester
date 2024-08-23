use crate::default_environment::*;
use crate::events::SolidityLikeEvent;
use crate::hashmap_based_memory::SimpleHashmapMemory;
use crate::simple_witness_tracer::MemoryLogWitnessTracer;
use crate::utils::IntoFixedLengthByteIterator;
use crate::{Address, H256, U256};
use std::collections::HashMap;
use std::hash::Hash;
use zk_evm::block_properties::*;
use zk_evm::reference_impls::decommitter::SimpleDecommitter;
use zk_evm::reference_impls::event_sink::{EventMessage, InMemoryEventSink};
use zk_evm::testing::storage::InMemoryStorage;
use zk_evm::vm_state::*;
use zk_evm::zk_evm_abstractions::precompiles::DefaultPrecompilesProcessor;
use zk_evm::zkevm_opcode_defs::decoding::AllowedPcOrImm;
use zk_evm::zkevm_opcode_defs::decoding::VmEncodingMode;
use zk_evm::zkevm_opcode_defs::definitions::ret::RET_IMPLICIT_RETURNDATA_PARAMS_REGISTER;
use zk_evm::zkevm_opcode_defs::system_params::{
    DEPLOYER_SYSTEM_CONTRACT_ADDRESS, DEPLOYER_SYSTEM_CONTRACT_ADDRESS_LOW,
    KNOWN_CODE_FACTORY_SYSTEM_CONTRACT_ADDRESS,
};
use zk_evm::zkevm_opcode_defs::{
    BlobSha256Format, ContractCodeSha256Format, FatPointer, VersionedHashLen32,
};
use zk_evm::{aux_structures::*, GenericNoopTracer};

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

        let range_end = if end_cap == 0 || end_word == 0 {
            end_word
        } else {
            end_word - 1
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

pub fn calldata_to_aligned_data(calldata: &[u8]) -> Vec<U256> {
    if calldata.is_empty() {
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
        hasher.update(buffer);

        let result = hasher.finalize();

        let key = U256::from_big_endian(result.as_slice());

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
    pub deployed_contracts: HashMap<Address, Vec<u8>>,
    pub execution_result: VmExecutionResult,
    pub returndata_bytes: Vec<u8>,
    pub raw_events: Vec<EventMessage>,
    pub to_l1_messages: Vec<EventMessage>,
    pub events: Vec<SolidityLikeEvent>,
    pub serialized_events: String,
    pub num_cycles_used: usize,
    pub num_ergs_used: u32,
    pub published_sha256_blobs: HashMap<U256, Vec<U256>>,
}

#[derive(Debug)]
pub struct RawInMemoryStorage {
    pub values: HashMap<StorageKey, H256>,
    pub contracts: HashMap<Address, Vec<u8>>,
    pub factory_deps: HashMap<H256, Vec<u8>>,
}

pub fn default_entry_point_contract_address() -> Address {
    Address::from_low_u64_be(1234567u64)
}

///
/// Used for testing the compiler with a single contract.
///
#[allow(clippy::too_many_arguments)]
pub fn run_vm(
    test_name: String,
    bytecode: Vec<u8>,
    calldata: &[u8],
    storage: HashMap<StorageKey, H256>,
    storage_transient: HashMap<StorageKey, H256>,
    context: Option<VmExecutionContext>,
    vm_launch_option: VmLaunchOption,
    cycles_limit: usize,
    known_contracts: HashMap<U256, Vec<u8>>,
    known_sha256_blobs: HashMap<U256, Vec<U256>>,
    default_aa_code_hash: U256,
    evm_simulator_code_hash: U256,
) -> anyhow::Result<VmSnapshot> {
    let entry_address = default_entry_point_contract_address();
    let mut contracts: HashMap<Address, Vec<u8>> = HashMap::new();
    contracts.insert(entry_address, bytecode);
    run_vm_multi_contracts(
        test_name,
        contracts,
        calldata,
        storage,
        storage_transient,
        entry_address,
        context,
        vm_launch_option,
        cycles_limit,
        known_contracts,
        known_sha256_blobs,
        default_aa_code_hash,
        evm_simulator_code_hash,
    )
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
    let memory = SimpleHashmapMemory::default();
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

pub fn create_vm<const B: bool>(
    mut tools: ExtendedTestingTools<B>,
    block_properties: BlockProperties,
    context: VmExecutionContext,
    contracts: &HashMap<Address, Vec<[u8; 32]>>,
    known_contracts: HashMap<U256, Vec<[u8; 32]>>,
) -> (
    VmState<
        InMemoryStorage,
        SimpleHashmapMemory,
        InMemoryEventSink,
        DefaultPrecompilesProcessor<B>,
        SimpleDecommitter<B>,
        MemoryLogWitnessTracer,
        8,
        zk_evm::zkevm_opcode_defs::decoding::EncodingModeProduction,
    >,
    HashMap<U256, Vec<[u8; 32]>>,
) {
    use zk_evm::contract_bytecode_to_words;
    use zk_evm::utils::bytecode_to_code_hash_for_mode;
    // fill the decommitter

    let mut factory_deps: HashMap<U256, Vec<U256>> = HashMap::new();
    let mut reverse_lookup_for_bytecode = HashMap::new();

    for (_address, bytecode) in contracts.iter() {
        let bytecode_hash = bytecode_to_code_hash_for_mode::<
            8,
            zk_evm::zkevm_opcode_defs::decoding::EncodingModeProduction,
        >(bytecode)
        .unwrap();
        let bytecode_hash_as_u256 = U256::from_big_endian(bytecode_hash.as_slice());

        reverse_lookup_for_bytecode.insert(bytecode_hash_as_u256, bytecode.to_owned());

        // add to decommitter
        let bytecode_words = contract_bytecode_to_words(bytecode.as_slice());
        let _existing = factory_deps.insert(bytecode_hash_as_u256, bytecode_words);
    }

    for (bytecode_hash, bytecode) in known_contracts.into_iter() {
        let bytecode_words = contract_bytecode_to_words(bytecode.as_slice());
        let _ = factory_deps.insert(bytecode_hash, bytecode_words);
        reverse_lookup_for_bytecode.insert(bytecode_hash, bytecode);
    }

    let decommitter_els: Vec<_> = factory_deps.into_iter().collect();

    tools.decommittment_processor.populate(decommitter_els);

    let mut vm = VmState::empty_state(
        tools.storage,
        tools.memory,
        tools.event_sink,
        tools.precompiles_processor,
        tools.decommittment_processor,
        tools.witness_tracer,
        block_properties,
    );

    let initial_context = CallStackEntry {
        this_address: context.this_address,
        msg_sender: context.msg_sender,
        code_address: context.this_address,
        base_memory_page: MemoryPage(INITIAL_BASE_PAGE),
        code_page: MemoryPage(ENTRY_POINT_PAGE),
        sp: 0,
        pc: 0,
        exception_handler_location: <<zk_evm::zkevm_opcode_defs::decoding::EncodingModeProduction as VmEncodingMode<8>>::PcOrImm as AllowedPcOrImm>::max(),
        ergs_remaining: zk_evm::zkevm_opcode_defs::system_params::VM_INITIAL_FRAME_ERGS
            - 0x80000000,
        this_shard_id: 0,
        caller_shard_id: 0,
        code_shard_id: 0,
        is_static: false,
        is_local_frame: false,
        context_u128_value: context.u128_value,
        heap_bound: 0,
        aux_heap_bound: 0,
        total_pubdata_spent: PubdataCost(0),
        stipend: 0u32,
    };

    // we consider the tested code as a bootloader
    vm.push_bootloader_context(0, initial_context);
    vm.local_state.timestamp = INITIAL_TIMESTAMP;
    vm.local_state.memory_page_counter = INITIAL_MEMORY_COUNTER;
    vm.local_state.tx_number_in_block = context.transaction_index as u16;
    // (50 gwei(l1 gas price) * 17(l1 gas per pubdata byte)) / 250000000 (l2 base fee)

    (vm, reverse_lookup_for_bytecode)
}

///
/// Used for testing the compiler with multiple contracts.
///
#[allow(clippy::too_many_arguments)]
pub fn run_vm_multi_contracts(
    test_name: String,
    contracts: HashMap<Address, Vec<u8>>,
    calldata: &[u8],
    storage: HashMap<StorageKey, H256>,
    storage_transient: HashMap<StorageKey, H256>,
    entry_address: Address,
    context: Option<VmExecutionContext>,
    vm_launch_option: VmLaunchOption,
    cycles_limit: usize,
    known_contracts: HashMap<U256, Vec<u8>>,
    known_sha256_blobs: HashMap<U256, Vec<U256>>,
    default_aa_code_hash: U256,
    evm_simulator_code_hash: U256,
) -> anyhow::Result<VmSnapshot> {
    let contracts = contracts
        .into_iter()
        .map(|(address, bytecode)| {
            let bytecode = bytecode
                .chunks(32)
                .map(|word| word.try_into().unwrap())
                .collect();
            (address, bytecode)
        })
        .collect();
    let known_contracts = known_contracts
        .into_iter()
        .map(|(address, bytecode)| {
            let bytecode = bytecode
                .chunks(32)
                .map(|word| word.try_into().unwrap())
                .collect();
            (address, bytecode)
        })
        .collect();
    run_vm_multi_contracts_inner(
        test_name,
        contracts,
        calldata,
        storage,
        storage_transient,
        entry_address,
        context,
        vm_launch_option,
        cycles_limit,
        known_contracts,
        known_sha256_blobs,
        default_aa_code_hash,
        evm_simulator_code_hash,
    )
}

///
/// Used for testing the compiler with multiple contracts.
///
#[allow(clippy::too_many_arguments)]
fn run_vm_multi_contracts_inner(
    _test_name: String,
    contracts: HashMap<Address, Vec<[u8; 32]>>,
    calldata: &[u8],
    storage: HashMap<StorageKey, H256>,
    storage_transient: HashMap<StorageKey, H256>,
    entry_address: Address,
    context: Option<VmExecutionContext>,
    vm_launch_option: VmLaunchOption,
    cycles_limit: usize,
    known_contracts: HashMap<U256, Vec<[u8; 32]>>,
    known_sha256_blobs: HashMap<U256, Vec<U256>>,
    default_aa_code_hash: U256,
    evm_simulator_code_hash: U256,
) -> anyhow::Result<VmSnapshot> {
    let (set_far_call_props, extra_props) = match &vm_launch_option {
        VmLaunchOption::Default => (true, None),
        VmLaunchOption::ManualCallABI(value) => (true, Some(value.clone())),
    };

    let mut tools = create_default_testing_tools();
    let mut block_properties = create_default_block_properties();
    block_properties.default_aa_code_hash = default_aa_code_hash;
    // we can always pretend it to be empty account
    block_properties.evm_simulator_code_hash = evm_simulator_code_hash;

    let calldata_length = calldata.len();

    // fill the calldata
    let aligned_calldata = calldata_to_aligned_data(calldata);
    let initial_bytecode = {
        let hash = storage
            .get(&StorageKey {
                address: Address::from_low_u64_be(DEPLOYER_SYSTEM_CONTRACT_ADDRESS_LOW.into()),
                key: U256::from_big_endian(entry_address.as_bytes()),
            })
            .ok_or_else(|| anyhow::anyhow!("Entry address code hash not found in the storage"))?;

        // If it's an EVM contract, we should run the EVM simulator
        if hash.as_bytes()[0] == BlobSha256Format::VERSION_BYTE {
            known_contracts
                .get(&evm_simulator_code_hash)
                .cloned()
                .ok_or_else(|| {
                    anyhow::anyhow!("EVM simulator bytecode not found in the known contracts")
                })?
        } else {
            contracts
                .get(&entry_address)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Initial bytecode not found"))?
        }
    };
    let initial_bytecode_as_memory = zk_evm::contract_bytecode_to_words(&initial_bytecode);

    tools.memory.populate(vec![
        (CALLDATA_PAGE, aligned_calldata),
        (ENTRY_POINT_PAGE, initial_bytecode_as_memory),
    ]);

    tools
        .decommittment_processor
        .populate(known_sha256_blobs.into_iter().collect());

    // fill the storage. Only rollup shard for now
    for (key, value) in storage.into_iter() {
        let per_address_entry = tools.storage.inner[0].entry(key.address).or_default();
        per_address_entry.insert(key.key, U256::from_big_endian(value.as_bytes()));
    }

    // fill the transient storage. Only rollup shard for now
    for (key, value) in storage_transient.into_iter() {
        let per_address_entry = tools.storage.inner_transient[0]
            .entry(key.address)
            .or_default();
        per_address_entry.insert(key.key, U256::from_big_endian(value.as_bytes()));
    }

    // some context notion
    let context = context.unwrap_or_else(|| VmExecutionContext {
        this_address: entry_address,
        ..Default::default()
    });

    // fill the rest
    let (mut vm, reverse_lookup_for_bytecode) = create_vm::<false>(
        tools,
        block_properties,
        context,
        &contracts,
        known_contracts,
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
    vm.witness_tracer.is_dummy = true;
    let mut tracer = GenericNoopTracer::new();
    for _ in 0..cycles_limit {
        vm.cycle(&mut tracer)?;
        super::evm_deploy::record_deployed_evm_bytecode(&mut vm);
        cycles_used += 1;

        // early return
        if let Some(end_result) = vm_may_have_ended(&vm) {
            result = Some(end_result);
            break;
        }
    }

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
        storage,
        event_sink,
        decommittment_processor,
        ..
    } = vm;

    let mut result_storage = HashMap::new();
    let mut deployed_contracts = HashMap::new();

    let (_full_history, raw_events, l1_messages) = event_sink.flatten();
    let events = crate::events::merge_events(raw_events.clone());

    let storage = storage.inner;
    let storage = storage.into_iter().next().unwrap();
    let mut published_sha256_blobs = HashMap::new();

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
                if let Some(bytecode) = reverse_lookup_for_bytecode.get(&value) {
                    deployed_contracts.insert(
                        deployed_address,
                        bytecode.iter().copied().flatten().collect(),
                    );
                }
            }

            let mut key_buffer = [0u8; 32];
            key.to_big_endian(&mut key_buffer);

            // This is an EVM blob hash that has been set as known.
            if address == *KNOWN_CODE_FACTORY_SYSTEM_CONTRACT_ADDRESS
                && value == 1.into()
                && key_buffer[0] == BlobSha256Format::VERSION_BYTE
            {
                let (_, normalized_hash) =
                    ContractCodeSha256Format::normalize_for_decommitment(&key_buffer);

                published_sha256_blobs.insert(
                    key,
                    decommittment_processor
                        .get_preimage_by_hash(normalized_hash)
                        .ok_or_else(|| anyhow::anyhow!("Published hash is unknown"))?
                        .clone(),
                );
            }
        }
    }

    // memory dump for returndata
    let returndata_page_content = vec![];

    let returndata_mem = MemoryArea {
        words: returndata_page_content,
    };

    let calldata_page_content = vec![];

    let calldata_mem = MemoryArea {
        words: calldata_page_content,
    };

    let returndata_bytes = match &execution_result {
        VmExecutionResult::Ok(ref res) => res.clone(),
        VmExecutionResult::Revert(ref res) => res.clone(),
        VmExecutionResult::Panic => vec![],
        VmExecutionResult::MostLikelyDidNotFinish(..) => vec![],
    };

    let compiler_tests_events: Vec<crate::events::Event> =
        events.iter().cloned().map(|el| el.into()).collect();

    let serialized_events = serde_json::to_string_pretty(&compiler_tests_events).unwrap();

    let did_call_or_ret_recently = local_state.previous_code_memory_page.0
        != local_state.callstack.get_current_stack().code_page.0;

    Ok(VmSnapshot {
        registers: local_state.registers,
        flags: local_state.flags,
        timestamp: local_state.timestamp,
        memory_page_counter: local_state.memory_page_counter,
        tx_number_in_block: local_state.tx_number_in_block,
        previous_super_pc: local_state.previous_super_pc.as_u64() as u32,
        did_call_or_ret_recently,
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
        // All the ergs from the empty frame should be passed into the root(bootloader) and unused ergs will be returned.
        num_ergs_used: zk_evm::zkevm_opcode_defs::system_params::VM_INITIAL_FRAME_ERGS
            - local_state.callstack.current.ergs_remaining,
        published_sha256_blobs,
    })
}

pub(crate) fn vm_may_have_ended<const B: bool>(
    vm: &VmState<
        InMemoryStorage,
        SimpleHashmapMemory,
        InMemoryEventSink,
        DefaultPrecompilesProcessor<B>,
        SimpleDecommitter<B>,
        MemoryLogWitnessTracer,
        8,
        zk_evm::zkevm_opcode_defs::decoding::EncodingModeProduction,
    >,
) -> Option<VmExecutionResult> {
    let execution_has_ended = vm.execution_has_ended();

    let r1 = vm.local_state.registers[RET_IMPLICIT_RETURNDATA_PARAMS_REGISTER as usize];
    let current_address = vm.local_state.callstack.get_current_stack().this_address;

    let outer_eh_location = <<zk_evm::zkevm_opcode_defs::decoding::EncodingModeProduction as VmEncodingMode<8>>::PcOrImm as AllowedPcOrImm>::max().as_u64();
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

pub(crate) fn dump_memory_page_using_primitive_value(
    memory: &SimpleHashmapMemory,
    ptr: PrimitiveValue,
) -> Vec<u8> {
    if !ptr.is_pointer {
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

pub(crate) fn dump_memory_page_by_offset_and_length(
    memory: &SimpleHashmapMemory,
    page: u32,
    offset: usize,
    length: usize,
) -> Vec<u8> {
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
        let it = word.into_be_iter();
        if is_first {
            is_first = false;
            let it = it.skip(unalignment);
            for next in it {
                if remaining > 0 {
                    dump.push(next);
                    remaining -= 1;
                }
            }
        } else {
            for next in it {
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
