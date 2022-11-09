use super::*;

mod add;
mod debug;
mod external;
mod ret;
mod uma;

use crate::runners::compiler_tests::VmLaunchOption;
use crate::runners::compiler_tests::VmTracingOptions;
use std::collections::HashMap;
use zk_evm::ethereum_types::U256;
use zkevm_assembly::Assembly;

pub(crate) fn run_for_result_only(assembly_text: &str) {
    use crate::runners::compiler_tests::*;

    use futures::executor::block_on;
    let assembly = Assembly::try_from(assembly_text.to_owned()).unwrap();
    let bytecode = assembly.clone().compile_to_bytecode().unwrap();
    let hash = U256::from(zk_evm::utils::bytecode_to_code_hash(&bytecode).unwrap());
    let mut known_contracts = HashMap::new();
    known_contracts.insert(hash, assembly.clone());

    let snapshot = block_on(run_vm(
        "manual".to_owned(),
        assembly.clone(),
        vec![],
        HashMap::new(),
        None,
        VmLaunchOption::Default,
        u16::MAX as usize,
        known_contracts,
        U256::zero(),
    )).unwrap();

    assert!(snapshot.execution_has_ended);
    assert!(
        matches!(snapshot.execution_result, VmExecutionResult::Ok(..)),
        "expected execution result Ok, found {:?}",
        snapshot.execution_result
    );
}
