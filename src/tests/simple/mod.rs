mod add;
mod debug;
mod external;
mod ret;
mod uma;

use std::collections::HashMap;
use zk_evm::ethereum_types::U256;
use zkevm_assembly::Assembly;

pub(crate) fn run_for_result_only(assembly_text: &str) {
    use crate::runners::compiler_tests::*;

    let assembly = Assembly::try_from(assembly_text.to_owned()).unwrap();
    let bytecode = assembly.clone().compile_to_bytecode().unwrap();
    let hash = U256::from(zk_evm::utils::bytecode_to_code_hash(&bytecode).unwrap());
    let mut known_contracts = HashMap::new();
    known_contracts.insert(hash, assembly.clone());
    let mut default_aa_placeholder_hash = [0u8; 32];
    default_aa_placeholder_hash[1] = 0x01; // to pass well-formedness check

    let snapshot = run_vm(
        "manual".to_owned(),
        assembly.clone(),
        &[],
        HashMap::new(),
        None,
        VmLaunchOption::Default,
        u16::MAX as usize,
        known_contracts,
        U256::from_big_endian(&default_aa_placeholder_hash),
        U256::from_big_endian(&default_aa_placeholder_hash),
    )
    .unwrap();

    assert!(snapshot.execution_has_ended);
    assert!(
        matches!(snapshot.execution_result, VmExecutionResult::Ok(..)),
        "expected execution result Ok, found {:?}",
        snapshot.execution_result
    );
}
