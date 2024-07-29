use super::*;
use zk_evm::testing::*;
use zk_evm::zk_evm_abstractions::precompiles::DefaultPrecompilesProcessor;

pub const INITIAL_TIMESTAMP: u32 = 8;
pub const INITIAL_MEMORY_COUNTER: u32 = 8;
pub const CALLDATA_PAGE: u32 = 3;
pub const INITIAL_BASE_PAGE: u32 = 4;
pub const ENTRY_POINT_PAGE: u32 = code_page_candidate_from_base(MemoryPage(INITIAL_BASE_PAGE)).0;
pub const DEFAULT_CALLER: &str = "3000";
pub const DEFAULT_CALLEE: &str = "5000";
pub const EMPTY_CONTEXT_HEX: &str = "0x0000000000000000000000000000000000000000";
pub const DEFAULT_CALLEE_HEX: &str = "0x0000000000000000000000000000000000001388";

pub fn default_callee_address() -> Address {
    let bytes: [u8; 20] = hex::decode(&DEFAULT_CALLEE_HEX[2..])
        .unwrap()
        .try_into()
        .unwrap();
    Address::from_slice(&bytes)
}

pub fn address_from_str_radix(str: &str, radix: u32) -> Address {
    use num_traits::Num;
    let value = num_bigint::BigUint::from_str_radix(str, radix).unwrap();
    let be_bytes = value.to_bytes_be();
    if be_bytes.len() > 20 {
        panic!("Address is too long");
    }

    let mut new = Address::default();
    new.as_bytes_mut()[(20 - be_bytes.len())..].copy_from_slice(&be_bytes);

    new
}

pub fn create_default_block_properties() -> BlockProperties {
    BlockProperties {
        default_aa_code_hash: U256::zero(),
        zkporter_is_available: true,
        evm_simulator_code_hash: U256::zero(),
    }
}

pub fn create_vm_with_default_settings<const B: bool>(
    tools: BasicTestingTools<B>,
    block_properties: BlockProperties,
) -> VmState<
    InMemoryStorage,
    SimpleMemory,
    InMemoryEventSink,
    DefaultPrecompilesProcessor<B>,
    SimpleDecommitter<B>,
    DummyTracer,
> {
    let mut vm = VmState::empty_state(
        tools.storage,
        tools.memory,
        tools.event_sink,
        tools.precompiles_processor,
        tools.decommittment_processor,
        tools.witness_tracer,
        block_properties,
    );

    let bootloader_context = CallStackEntry {
        this_address: address_from_str_radix(DEFAULT_CALLEE, 10),
        msg_sender: address_from_str_radix(DEFAULT_CALLER, 10),
        code_address: address_from_str_radix(DEFAULT_CALLER, 10),
        base_memory_page: MemoryPage(INITIAL_BASE_PAGE),
        code_page: MemoryPage(ENTRY_POINT_PAGE),
        sp: 0u16,
        pc: 0u16,
        exception_handler_location: 0u16,
        ergs_remaining: zk_evm::zkevm_opcode_defs::system_params::VM_INITIAL_FRAME_ERGS,
        this_shard_id: 0,
        caller_shard_id: 0,
        code_shard_id: 0,
        is_static: false,
        is_local_frame: false,
        context_u128_value: 0,
        heap_bound: 0,
        aux_heap_bound: 0,
        total_pubdata_spent: PubdataCost(0),
        stipend: 0u32,
    };

    // we consider the tested code as a bootloader
    vm.push_bootloader_context(0, bootloader_context);
    vm.local_state.timestamp = INITIAL_TIMESTAMP;
    vm.local_state.memory_page_counter = INITIAL_MEMORY_COUNTER;

    vm
}
