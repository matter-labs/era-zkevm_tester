use super::*;

use zk_evm::opcodes::execution::far_call::*;
use zk_evm::precompiles::DEPLOYER_PRECOMPILE_ADDRESS;
use zk_evm::testing::simple_tracer::NoopTracer;
use zkevm_assembly::Assembly;
use crate::{U256, Address, H256};
use std::collections::HashMap;
use std::hash::Hash;
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
    )
    .await
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

    let bootloader_context = CallStackEntry {
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
    vm.push_bootloader_context(bootloader_context);
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
    tools.memory.populate(vec![(CALLDATA_PAGE, aligned_calldata)]);

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
    
    for _ in 0..cycles_limit {
        vm.cycle(&mut NoopTracer);
    }

    // TODO: dump returndata

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
        _ => unreachable!()
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

    VmSnapshot {
        registers: local_state.registers,
        flags: local_state.flags,
        timestamp: local_state.timestamp,    
        memory_page_counter: local_state.memory_page_counter,    
        tx_number_in_block: local_state.tx_number_in_block,    
        previous_pc: local_state.previous_pc, 
        did_call_or_ret_recently: local_state.did_call_or_ret_recently, 
        tx_origin: local_state.tx_origin, 
        calldata_area_dump: MemoryArea::empty(),
        returndata_area_dump: MemoryArea::empty(),
        execution_has_ended,
        stack_dump: MemoryArea::empty(),
        heap_dump: MemoryArea::empty(),
        storage: result_storage,
        deployed_contracts,
        execution_result,
    }
}