#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_no_overflow_set_gt() {
        let assembly: &'static str = r#"
                .text
                .rodata.cst32
                .p2align	5
            CPI0_0:
                .cell 16777184
            CPI0_1:
                .cell 16777152
                .text
                .globl	__entry
            __entry:
            .func_begin0:
                add 1, r0, r1
                add 2, r0, r2
                add! r1, r2, r0
                jump.gt @.test_success
            .test_panic:
                ret.panic r0
            .test_success:
                ret.ok r0
            .note.GNU-stack
                "#;

        run_for_result_only(assembly);
    }
}
