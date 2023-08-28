pub(crate) fn read_assembly_from_file(path_and_filename: &str) -> String {
    let base_path = std::path::Path::new("./src/tests/simple/external/");
    let full_path = base_path.join(path_and_filename);
    let content = std::fs::read_to_string(full_path).unwrap();

    content
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        runners::compiler_tests::{set_tracing_mode, VmLaunchOption, VmTracingOptions},
        trace::run_inner,
    };

    #[test]
    fn test_simple_storage() {
        set_tracing_mode(VmTracingOptions::ManualVerbose);
        let assembly = read_assembly_from_file("solidity_by_example/simple/first_app.sol.asm");
        run_inner(
            &hex::decode("06661abd").unwrap(),
            VmLaunchOption::Default,
            &assembly,
        );
    }
}
