use super::*;

mod add;
mod uma;
mod debug;

use zkevm_assembly::Assembly;
use std::collections::HashMap;
use crate::runners::compiler_tests::VmLaunchOption;
use crate::runners::compiler_tests::VmTracingOptions;

pub(crate) fn run_for_result_only(assembly_text: &str) {
    use crate::runners::compiler_tests::*;

    use futures::executor::block_on;
    let assembly = Assembly::try_from(assembly_text.to_owned()).unwrap();
    let snapshot = block_on(run_vm(
        assembly.clone(),
        vec![],
        HashMap::new(),
        vec![],
        None,
        VmLaunchOption::Default,
        1024,
        u16::MAX as usize,
        vec![assembly.clone()],
        vec![],
        HashMap::new(),
    ));

    assert!(snapshot.execution_has_ended);
    assert!(matches!(snapshot.execution_result, VmExecutionResult::Ok(..)));
}