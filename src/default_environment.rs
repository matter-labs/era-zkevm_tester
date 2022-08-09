use super::*;
use zk_evm::{testing::*, zkevm_opcode_defs::decoding::EncodingModeProduction};

pub const INITIAL_TIMESTAMP: u32 = 8;
pub const INITIAL_MEMORY_COUNTER: u32 = 8;
pub const CALLDATA_PAGE: u32 = 3;
pub const INITIAL_BASE_PAGE: u32 = 4;
pub const ENTRY_POINT_PAGE: u32 =
    CallStackEntry::<8, EncodingModeProduction>::code_page_candidate_from_base(MemoryPage(
        INITIAL_BASE_PAGE,
    ))
    .0;
pub const DEFAULT_CALLER: &'static str = "3000";
pub const DEFAULT_CALLEE: &'static str = "5000";
pub const EMPTY_CONTEXT_HEX: &'static str = "0x0000000000000000000000000000000000000000";
pub const DEFAULT_CALLEE_HEX: &'static str = "0x0000000000000000000000000000000000001388";

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
        ergs_per_code_decommittment_word: 1,
        zkporter_is_available: true,
    }
}

pub fn create_vm_with_default_settings<'a, const B: bool>(
    tools: &'a mut BasicTestingTools<B>,
    block_properties: &'a BlockProperties,
) -> VmState<
    'a,
    InMemoryStorage,
    SimpleMemory,
    InMemoryEventSink,
    DefaultPrecompilesProcessor<B>,
    SimpleDecommitter<B>,
    DummyTracer,
> {
    let mut vm = VmState::empty_state(
        &mut tools.storage,
        &mut tools.memory,
        &mut tools.event_sink,
        &mut tools.precompiles_processor,
        &mut tools.decommittment_processor,
        &mut tools.witness_tracer,
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
        ergs_remaining: u32::MAX,
        this_shard_id: 0,
        caller_shard_id: 0,
        code_shard_id: 0,
        is_static: false,
        is_local_frame: false,
        context_u128_value: 0,
    };

    // we consider the tested code as a bootloader
    vm.push_bootloader_context(0, bootloader_context);
    vm.local_state.timestamp = INITIAL_TIMESTAMP;
    vm.local_state.memory_page_counter = INITIAL_MEMORY_COUNTER;

    vm
}
