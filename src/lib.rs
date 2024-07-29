#![allow(clippy::type_complexity)]

use zk_evm::aux_structures::*;
use zk_evm::block_properties::BlockProperties;
use zk_evm::reference_impls::decommitter::SimpleDecommitter;
use zk_evm::reference_impls::event_sink::InMemoryEventSink;
use zk_evm::reference_impls::memory::SimpleMemory;
use zk_evm::testing::storage::InMemoryStorage;
use zk_evm::vm_state::*;
use zk_evm::witness_trace::DummyTracer;
use zk_evm::zkevm_opcode_defs::ethereum_types::*;

pub mod compiler_tests;
pub mod default_environment;
pub mod events;
pub mod evm_deploy;
pub mod hashmap_based_memory;
pub mod simple_witness_tracer;
pub mod utils;
