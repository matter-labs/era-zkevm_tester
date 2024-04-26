use std::fs::File;

use serde_json::json;
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

pub(crate) fn publish_evm_bytecode_interface() -> ethabi::Contract {
    let known_code_storage_abi = json!(
        [
            {
                "inputs": [
                  {
                    "internalType": "bytes",
                    "name": "bytecode",
                    "type": "bytes"
                  }
                ],
                "name": "publishEVMBytecode",
                "outputs": [],
                "stateMutability": "nonpayable",
                "type": "function"
            }
        ]
    );

    serde_json::from_value(known_code_storage_abi).unwrap()
}
