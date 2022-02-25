use super::*;

use zk_evm::opcodes::execution::far_call::*;
use zk_evm::opcodes::execution::ret::*;
use zk_evm::precompiles::DEPLOYER_PRECOMPILE_ADDRESS;
use zk_evm::testing::simple_tracer::NoopTracer;
use zkevm_assembly::Assembly;
use crate::{U256, Address, H256};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::atomic::AtomicBool;
use zk_evm::testing::*;
use crate::default_environment::*;
use zk_evm::block_properties::*;
use zk_evm::testing::storage::InMemoryStorage;
use zk_evm::testing::memory::SimpleMemory;
use zk_evm::testing::event_sink::InMemoryEventSink;
use zk_evm::precompiles::DefaultPrecompilesProcessor;
use zk_evm::testing::decommitter::SimpleDecommitter;
use zk_evm::witness_trace::DummyTracer;
use zk_evm::vm_state::*;
use zk_evm::aux_structures::*;

use sha2::{Sha256, Digest};

#[derive(Debug)]
pub enum VmLaunchOption {
    Default,
    Pc(u16),
    Label(String),
}

#[derive(Debug)]
pub enum VmExecutionResult {
    Ok(Vec<u8>),
    Revert(Vec<u8>),
    Panic,
    MostLikelyDidNotFinish(Address, u16)
}

#[derive(Debug, Default)]
pub struct VmExecutionContext {
    this_address: Address,
    msg_sender: Address,
    block_number: u64,
    transaction_index: u32,
    block_timestamp: u64,
}

impl VmExecutionContext {
    pub fn new(
        this_address: Address,
        msg_sender: Address,
        block_number: u64,
        transaction_index: u32,
        block_timestamp: u64,
    ) -> Self {
        Self {
            this_address,
            msg_sender,
            block_number,
            transaction_index,
            block_timestamp,
        }
    }
}

#[derive(Debug)]
pub struct MemoryArea {
    pub words: Vec<U256>
}

impl MemoryArea {
    pub fn empty() -> Self {
        Self {
            words: vec![]
        }
    }   

    pub fn dump_be_bytes(&self, range: std::ops::Range<usize>) -> Vec<u8> {
        if range.is_empty() {
            return vec![];
        }

        let mut result = Vec::with_capacity(range.len());

        let starting_word = range.start % 32;
        let start_bytes = range.start / 32;
        if start_bytes != 0 {
            let el = self.words.get(starting_word).copied().unwrap_or(U256::zero());
            let mut buffer = [0u8; 32];
            el.to_big_endian(&mut buffer);
            result.extend_from_slice(&buffer[(32-start_bytes)..]);
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

pub fn hash_contract_code(code: &Vec<[u8; 32]>) -> H256 {
    let mut hasher = Sha256::new();
    for code_word in code.iter() {
        hasher.update(code_word);
    }

    H256::from_slice(hasher.finalize().as_slice())
}

pub fn contract_bytecode_to_words(code: Vec<[u8; 32]>) -> Vec<U256> {
    code.into_iter().map(|el| U256::from_little_endian(&el)).collect()
}

pub fn calldata_to_aligned_data(calldata: &Vec<u8>) -> Vec<U256> {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StorageKey {
    pub address: Address,
    pub key: U256
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
}

#[derive(Debug)]
pub struct VmSnapshot {
    pub registers: [U256; zk_evm::zkevm_opcode_defs::REGISTERS_COUNT],
    pub flags: zk_evm::flags::Flags,
    pub timestamp: u32,    
    pub memory_page_counter: u32,
    pub tx_number_in_block: u16,
    pub previous_pc: u16,
    pub did_call_or_ret_recently: bool,
    pub tx_origin: Address, 
    pub calldata_area_dump: MemoryArea,
    pub returndata_area_dump: MemoryArea,
    pub execution_has_ended: bool,
    pub stack_dump: MemoryArea,
    pub heap_dump: MemoryArea,
    pub storage: HashMap<StorageKey, H256>,
    pub deployed_contracts: HashMap<Address, Assembly>,
    pub execution_result: VmExecutionResult,
    pub returndata_bytes: Vec<u8>
}

#[derive(Debug)]
pub struct RawInMemoryStorage {
    pub values: HashMap<StorageKey, H256>,
    pub contracts: HashMap<Address, Assembly>,
    pub factory_deps: HashMap<H256, Vec<u8>>,
}

///
/// Used for testing the compiler with a single contract.
///
#[allow(clippy::too_many_arguments)]
pub async fn run_vm(
    assembly: Assembly,
    calldata: Vec<u8>,
    storage: HashMap<StorageKey, H256>,
    registers: Vec<U256>,
    context: Option<VmExecutionContext>,
    vm_launch_option: VmLaunchOption,
    cycles_limit: usize,
    max_stack_size: usize,
    known_contracts: Vec<Assembly>,
    known_bytecodes: Vec<Vec<[u8; 32]>>,
    factory_deps: HashMap<H256, Vec<[u8; 32]>>,
) -> VmSnapshot {
    let mut contracts: HashMap<Address, Assembly> = HashMap::new();
    contracts.insert(Address::default(), assembly);
    run_vm_multi_contracts(
        contracts,
        calldata,
        storage,
        registers,
        Address::default(),
        context,
        vm_launch_option,
        cycles_limit,
        max_stack_size,
        known_contracts,
        known_bytecodes,
        factory_deps,
    ).await
}


pub fn create_vm<
    'a, const B: bool
>(
    tools: &'a mut BasicTestingTools<B>, 
    block_properties: &'a BlockProperties,
    context: VmExecutionContext,
    registers: Vec<U256>,
    contracts: &HashMap<Address, Assembly>,
    known_contracts: Vec<Assembly>,
    known_bytecodes: Vec<Vec<[u8; 32]>>,
    factory_deps: HashMap<H256, Vec<[u8; 32]>>,
    initial_pc: u16,
) -> (VmState<'a, InMemoryStorage, SimpleMemory, InMemoryEventSink, DefaultPrecompilesProcessor<B>, SimpleDecommitter<B>, DummyTracer>,
    HashMap<U256, Assembly>)
{
    // fill the decommitter and storage slots with contract codes, etc

    // first deployed contracts. Those are stored under DEPLOYER_CONTRACT as raw address -> hash
    let mut storage_els = vec![];
    let mut factory_deps: HashMap<U256, Vec<U256>> = factory_deps.into_iter().map(|(k, v)| {
        (U256::from_big_endian(k.as_bytes()), contract_bytecode_to_words(v))
    }).collect();

    let mut reverse_lookup_for_assembly = HashMap::new();

    for (address, assembly) in contracts.iter() {
        let bytecode = assembly.clone().compile_to_bytecode().expect("must compile an assembly");
        let bytecode_hash = hash_contract_code(&bytecode);
        let key = U256::from_big_endian(address.as_bytes());
        let value = U256::from_big_endian(bytecode_hash.as_bytes());
    
        reverse_lookup_for_assembly.insert(value, assembly.clone());

        // add to decommitter
        let bytecode_words = contract_bytecode_to_words(bytecode);
        let _existing = factory_deps.insert(value, bytecode_words);

        storage_els.push((0, *address, key, value));
    }

    for assembly in known_contracts.into_iter() {
        let bytecode = assembly.compile_to_bytecode().expect("must compile an assembly");;
        let bytecode_hash = hash_contract_code(&bytecode);
        let bytecode_words = contract_bytecode_to_words(bytecode);
        let _ = factory_deps.insert(U256::from_big_endian(bytecode_hash.as_bytes()), bytecode_words);
    }

    for bytecode in known_bytecodes.into_iter() {
        let bytecode_hash = hash_contract_code(&bytecode);
        let bytecode_words = contract_bytecode_to_words(bytecode);
        let _ = factory_deps.insert(U256::from_big_endian(bytecode_hash.as_bytes()), bytecode_words);
    }

    let decommitter_els: Vec<_> = factory_deps.into_iter().into_iter().collect();

    tools.decommittment_processor.populate(decommitter_els);
    tools.storage.populate(storage_els);

    let mut vm = VmState::empty_state(
        &mut tools.storage, 
        &mut tools.memory, 
        &mut tools.event_sink, 
        &mut tools.precompiles_processor, 
        &mut tools.decommittment_processor, 
        &mut tools.witness_tracer, 
        block_properties
    );

    for (i, value) in registers.into_iter().enumerate() {
        vm.perform_dst1_update(value, (i+1) as u8);
    }

    let initial_context = CallStackEntry {
        this_address: context.this_address,
        msg_sender: context.msg_sender,
        code_address: context.this_address,
        base_memory_page: MemoryPage(INITIAL_BASE_PAGE),
        code_page: MemoryPage(ENTRY_POINT_PAGE),
        calldata_page: MemoryPage(CALLDATA_PAGE),
        returndata_page: MemoryPage(0),
        sp: 0u16,
        pc: initial_pc,
        exception_handler_location: u16::MAX,
        ergs_remaining: u32::MAX,
        this_shard_id: 0,
        caller_shard_id: 0,
        code_shard_id: 0,
        is_static: false,
        is_local_frame: false,
    };

    // we consider the tested code as a bootloader
    vm.push_bootloader_context(initial_context);
    vm.local_state.timestamp = INITIAL_TIMESTAMP;
    vm.local_state.memory_page_counter = INITIAL_MEMORY_COUNTER;
    vm.local_state.tx_number_in_block = context.transaction_index as u16;


    (vm, reverse_lookup_for_assembly)
}

///
/// Used for testing the compiler with multiple contracts.
///
#[allow(clippy::too_many_arguments)]
pub async fn run_vm_multi_contracts(
    contracts: HashMap<Address, Assembly>,
    calldata: Vec<u8>,
    storage: HashMap<StorageKey, H256>,
    registers: Vec<U256>,
    entry_address: Address,
    context: Option<VmExecutionContext>,
    vm_launch_option: VmLaunchOption,
    cycles_limit: usize,
    _max_stack_size: usize,
    known_contracts: Vec<Assembly>,
    known_bytecodes: Vec<Vec<[u8; 32]>>,
    factory_deps: HashMap<H256, Vec<[u8; 32]>>,
) -> VmSnapshot {
    let initial_pc = match vm_launch_option {
        VmLaunchOption::Pc(pc) => pc,
        VmLaunchOption::Label(label) => {
            let offset = *contracts
            .get(&entry_address)
            .unwrap()
            .function_labels
            .get(&label)
            .unwrap();

            assert!(offset <= u16::MAX as usize);

            offset as u16
        },
        VmLaunchOption::Default => 0,
    };

    let mut tools = create_default_testing_tools();
    let mut block_properties = create_default_block_properties();

    // fill the calldata
    let aligned_calldata = calldata_to_aligned_data(&calldata);
    // and initial memory page
    let initial_assembly = contracts.get(&entry_address).cloned().unwrap();
    let initial_bytecode = initial_assembly.clone().compile_to_bytecode().unwrap();
    let initial_bytecode_as_memory = contract_bytecode_to_words(initial_bytecode);

    tools.memory.populate(vec![
        (CALLDATA_PAGE, aligned_calldata),
        (ENTRY_POINT_PAGE, initial_bytecode_as_memory)
        ]
    );

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

    // use block-global data from context
    block_properties.block_number = context.block_number;
    block_properties.block_timestamp = context.block_timestamp;

    // fill the rest
    let (mut vm, reverse_lookup_for_assembly) = create_vm(
        &mut tools,
        &block_properties,
        context,
        registers,
        &contracts,
        known_contracts,
        known_bytecodes,
        factory_deps,
        initial_pc
    );

    let debug = get_debug();
    
    for _ in 0..cycles_limit {
        if debug {
            let mut tracer = super::debug_tracer::DebugTracerWithAssembly{assembly: &initial_assembly};
            vm.cycle(&mut tracer);
        } else {
            vm.cycle(&mut NoopTracer);
        }
    }

    let r1 = vm.local_state.registers[RET_IMPLICIT_RETURNDATA_OFFSET_REGISTER as usize];
    let r2 = vm.local_state.registers[RET_IMPLICIT_RETURNDATA_LENGTH_REGISTER as usize];

    let returndata_offset = r1.0[0] as usize;
    let returndata_length = r2.0[0] as usize;
    let returndata_page = vm.local_state.callstack.get_current_stack().returndata_page;
    let current_address = vm.local_state.callstack.get_current_stack().this_address;

    let execution_result = match (vm.local_state.flags.overflow_or_less_than_flag, vm.local_state.callstack.get_current_stack().pc) {
        (false, 0) => {
            VmExecutionResult::Ok(vec![])
        },
        (false, u16::MAX) => {
            VmExecutionResult::Revert(vec![])
        },
        (true, u16::MAX) => {
            VmExecutionResult::Panic
        },
        (_, a) => {
            VmExecutionResult::MostLikelyDidNotFinish(current_address, a)
        }
    };

    let execution_has_ended = vm.execution_has_ended();

    let VmState { 
        local_state, 
        block_properties: _,
        ..
    } = vm;

    let mut result_storage = HashMap::new();
    let mut deployed_contracts = HashMap::new();

    let BasicTestingTools {
        storage,
        memory,
        event_sink: _,
        precompiles_processor: _,
        decommittment_processor: _,
        witness_tracer: _,
    } = tools;

    let storage = storage.inner;
    let storage = storage.into_iter().next().unwrap();

    for (address, inner) in storage.into_iter() {
        for (key, value) in inner.into_iter() {
            let storage_key = StorageKey {
                address,
                key
            };
            let mut buffer = [0u8; 32];
            value.to_big_endian(&mut buffer);
            let value_h256 = H256::from_slice(&buffer);
            result_storage.insert(storage_key, value_h256);

            if address == *DEPLOYER_PRECOMPILE_ADDRESS {
                let mut buffer = [0u8; 32];
                key.to_little_endian(&mut buffer);
                let deployed_address = Address::from_slice(&buffer[12..]);
                if let Some(known_assembly) = reverse_lookup_for_assembly.get(&value) {
                    deployed_contracts.insert(deployed_address, known_assembly.clone());
                }
            }
        }
    }

    // memory dump for returndata
    let returndata_page_content = memory.inner.get(&returndata_page.0).cloned().unwrap_or(vec![]);
    let returndata_mem = MemoryArea {
        words: returndata_page_content
    };

    let calldata_page_content = memory.inner.get(&CALLDATA_PAGE).cloned().unwrap_or(vec![]);
    let calldata_mem = MemoryArea {
        words: calldata_page_content
    };

    let returndata_bytes = returndata_mem.dump_be_bytes(returndata_offset..(returndata_offset + returndata_length));

    VmSnapshot {
        registers: local_state.registers,
        flags: local_state.flags,
        timestamp: local_state.timestamp,    
        memory_page_counter: local_state.memory_page_counter,    
        tx_number_in_block: local_state.tx_number_in_block,    
        previous_pc: local_state.previous_pc, 
        did_call_or_ret_recently: local_state.did_call_or_ret_recently, 
        tx_origin: local_state.tx_origin, 
        calldata_area_dump: calldata_mem,
        returndata_area_dump: returndata_mem,
        execution_has_ended,
        stack_dump: MemoryArea::empty(),
        heap_dump: MemoryArea::empty(),
        storage: result_storage,
        deployed_contracts,
        execution_result,
        returndata_bytes
    }
}

pub static USE_DEBUG: AtomicBool = AtomicBool::new(false);

pub fn set_debug(value: bool) {
    USE_DEBUG.store(value, std::sync::atomic::Ordering::SeqCst);
}

pub fn get_debug() -> bool {
    USE_DEBUG.load(std::sync::atomic::Ordering::Relaxed)
}

#[cfg(test)]
mod test {
    use core::time;

    use super::*;


    pub(crate) const TEST_ASSEMBLY_0: &'static str = r#"
        .text
        .file    "fib.ll"
        .rodata.cst32
        .p2align    5                               ; -- Begin function fn_fib
    CPI0_0:
        .cell -57896044618658097711785492504343953926634992332820282019728792003956564819968
        .text
        .globl    fn_fib
    fn_fib:                                 ; @fn_fib
    ; %bb.0:                                ; %fn_fib_entry
        nop stack+=[6]
        add    r1, r0, r2
        add    @CPI0_0[0], r0, r1
        add    0, r0, r4
        add.gt    r1, r0, r4
        add    0, 0, r3
        add    0, r0, r5
        add.lt    r1, r0, r5
        add.eq    r5, r0, r4
        sub!    r4, r3, r4
        jump.ne    @.BB0_2
    ; %bb.1:                                ; %fn_fib_entry.if
        add    1, 0, r4
        sub!    r2, r4, r4
        and    r2, r1, r2
        sub!    r2, r3, r3
        sub!    r2, r1, r1
        add    1, 0, r1
        nop stack-=[6]
        ret
    .BB0_2:                                 ; %fn_fib_entry.endif
        sub.s    1, r2, r1
        call    @fn_fib
        add    r1, r0, r3
        sub.s    2, r2, r1
        call    @fn_fib
        add    r3, r1, r1
        nop stack-=[6]
        ret
                                            ; -- End function
        .note.GNU-stack
        "#;


    #[test]
    fn test_fib() {
        use futures::executor::block_on;
        set_debug(true);

        let assembly = Assembly::try_from(TEST_ASSEMBLY_0.to_owned()).unwrap();

        let snapshot = block_on(run_vm(
            assembly.clone(),
            vec![],
            HashMap::new(),
            vec![U256::from_dec_str("5").unwrap()],
            None,
            VmLaunchOption::Default,
            11,
            u16::MAX as usize,
            vec![assembly.clone()],
            vec![],
            HashMap::new(),
        ));

        let VmSnapshot { registers, flags, timestamp, memory_page_counter, tx_number_in_block, previous_pc, did_call_or_ret_recently, tx_origin, calldata_area_dump, returndata_area_dump, execution_has_ended, stack_dump, heap_dump, storage, deployed_contracts, execution_result, returndata_bytes } = snapshot;
        dbg!(execution_has_ended);
        dbg!(execution_result);
        dbg!(registers);
        dbg!(timestamp);
    }
}