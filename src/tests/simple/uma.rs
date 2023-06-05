#[cfg(test)]
mod tests {
    use crate::runners::compiler_tests::set_tracing_mode;

    use super::*;

    #[test]
    fn uma_trivial_write() {
        let assembly: &'static str = r#"
                .text
                .rodata.cst32
                .p2align	5
            U256_MAX:
                .cell 115792089237316195423570985008687907853269984665640564039457584007913129639935
            U128_MAX:
                .cell 340282366920938463463374607431768211455
                .text
                .globl	__entry
            __entry:
            .func_begin0:
                add 16, r0, r1
                add @U256_MAX[0], r0, r2
                uma.heap_write r1, r2, r0
                uma.heap_read r1, r0, r3
                sub! r2, r3, r0
                jump.eq @.test_success
            .test_panic:
                ret.panic r0
            .test_success:
                ret.ok r0
            .note.GNU-stack
                "#;

        run_for_result_only(assembly);
    }

    #[test]
    fn uma_unaligned_write_read_original_high() {
        let assembly: &'static str = r#"
                .text
                .rodata.cst32
                .p2align	5
            U256_MAX:
                .cell 115792089237316195423570985008687907853269984665640564039457584007913129639935
            U128_MAX:
                .cell 340282366920938463463374607431768211455
                .text
                .globl	__entry
            __entry:
            .func_begin0:
                add 16, r0, r1
                add @U256_MAX[0], r0, r2
                uma.heap_write r1, r2, r0
                uma.heap_read r0, r0, r3
                add @U128_MAX[0], r0, r4
                sub! r3, r4, r0
                jump.eq @.test_success
            .test_panic:
                ret.panic r0
            .test_success:
                ret.ok r0
            .note.GNU-stack
                "#;

        // set_tracing_mode(VmTracingOptions::ManualVerbose);
        // crate::trace::run_inner(vec![], VmLaunchOption::Default, assembly);

        run_for_result_only(assembly);
    }

    #[test]
    fn uma_unaligned_write_read_original_low() {
        let assembly: &'static str = r#"
                .text
                .rodata.cst32
                .p2align	5
            U256_MAX:
                .cell 115792089237316195423570985008687907853269984665640564039457584007913129639935
            U128_MAX:
                .cell 340282366920938463463374607431768211455
            U128_MAX_SHIFTED_INTO_HIGH:
                .cell 115792089237316195423570985008687907852929702298719625575994209400481361428480
                .text
                .globl	__entry
            __entry:
            .func_begin0:
                add 16, r0, r1
                add @U256_MAX[0], r0, r2
                uma.heap_write r1, r2, r0
                add 32, r0, r1
                uma.heap_read r1, r0, r3
                add @U128_MAX_SHIFTED_INTO_HIGH[0], r0, r4
                sub! r3, r4, r0
                jump.eq @.test_success
            .test_panic:
                ret.panic r0
            .test_success:
                ret.ok r0
            .note.GNU-stack
                "#;

        run_for_result_only(assembly);
    }
}
