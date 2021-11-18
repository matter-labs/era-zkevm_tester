use ethereum_types::*;

use zk_evm::vm_state::*;
use zk_evm::aux_structures::*;

use zk_evm::testing::decommitter::SimpleDecommitter;
use zk_evm::precompiles::DefaultPrecompilesProcessor;
use zk_evm::testing::event_sink::InMemoryEventSink;
use zk_evm::testing::memory::SimpleMemory;
use zk_evm::testing::storage::InMemoryStorage;
use zk_evm::block_properties::BlockProperties;
use zk_evm::witness_trace::DummyTracer;
use zkevm_assembly::Assembly;

pub mod default_environment;
pub mod runner;

use zk_evm::opcodes::DecodedOpcode;
use zk_evm::flags::Flags;

#[derive(Debug, Clone)]
pub struct PartialVmState {
    pub skip_cycle: bool,
    pub error_flags_collection: ErrorFlags,
    pub final_masked_opcode: DecodedOpcode,
    pub resolved_jump_condition: bool,
    pub registers: [U256; zk_evm::zkevm_opcode_defs::REGISTERS_COUNT],
    pub flags: Flags,
    pub timestamp: u32,    
    pub memory_page_counter: u32,
    pub tx_number_in_block: u16,
    pub pending_port: LogPendingPort,
    pub pending_cycles_left: Option<usize>,
    pub tx_origin: Box<Address>, // large one
    pub callstack: Callstack,
}