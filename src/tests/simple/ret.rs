#[cfg(test)]
mod tests {
    use crate::tests::simple::run_for_result_only;

    #[test]
    fn test_ret_revert() {
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
                add 64, r0, r1
                add 64, r0, r2
                shl.s 32, r2, r2
                add r1, r2, r1
                ret.revert r1
            .note.GNU-stack
                "#;

        run_for_result_only(assembly);
    }
}
