use std::fs::File;

use zk_evm::zkevm_opcode_defs::ethereum_types::*;

use zk_evm::aux_structures::*;
use zk_evm::vm_state::*;

use zk_evm::block_properties::BlockProperties;
use zk_evm::reference_impls::decommitter::SimpleDecommitter;
use zk_evm::reference_impls::event_sink::InMemoryEventSink;
use zk_evm::reference_impls::memory::SimpleMemory;
use zk_evm::testing::storage::InMemoryStorage;
use zk_evm::witness_trace::DummyTracer;
use zkevm_assembly::Assembly;

mod evm_deploy_tracer;
pub mod default_environment;
pub mod runners;
pub mod trace;
mod utils;

mod tests;

pub(crate) fn read_known_code_storage() -> ethabi::Contract {
    let mut read_value: serde_json::Value = serde_json::from_reader(
        File::open("./src/abi/KnownCodesStorage.json").unwrap_or_else(|e| panic!("Failed to open KnownCodeStorage: {}", e)),
    )
    .unwrap_or_else(|e| panic!("Failed to parse KnownCodeStorage: {}", e));

    serde_json::from_value(read_value["abi"].take()).unwrap()
}
