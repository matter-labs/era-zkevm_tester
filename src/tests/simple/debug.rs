use super::*;

#[cfg(test)]
mod tests {
    use crate::trace::run_inner;
    use crate::runners::compiler_tests::set_tracing_mode;

    use super::*;

    #[test]
    fn test_manual() {
        let assembly: &'static str = r#"
        .rodata
        C0:
            .cell 7749745057451750595669064617574929170518332649783278685167248603617863602466
        .text
        .BB1_9:
        add     @C0[0], r0, r2
        add     4, r0, r3
        uma.heap_write  r3, r2, r0
        uma.heap_read r0, r0, r4
        add 32, r0, r5
        uma.heap_read r5, r0, r5
        add 2, r0, r4
        uma.heap_read r4, r0, r4
        add 34, r0, r5
        uma.heap_read r5, r0, r5
        add     36, r0, r6
.BB1_1:
        uma.heap_write  r2, r1, r0
        uma.heap_write  r2, r6, r0
        revert
                "#;

        set_tracing_mode(VmTracingOptions::ManualVerbose);
        run_inner(vec![], VmLaunchOption::Default, assembly);

        // run_for_result_only(assembly);
    }
}