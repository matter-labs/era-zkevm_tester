#[cfg(test)]
mod tests {
    use crate::runners::compiler_tests::{set_tracing_mode, VmLaunchOption, VmTracingOptions};
    use crate::trace::run_inner;

    #[test]
    fn test_manual() {
        //     let assembly: &'static str = r#"
        // 	.text
        // 	.file	"calldata_array_order.sol:Test"
        // 	.text.unlikely.
        // 	.globl	__entry
        // __entry:
        // 	add 256, r1, r1
        // 	add 64, r0, r2
        // 	add 128, r0, r3
        // 	add 256, r0, r4
        // 	st.1.inc r1, r2, r1
        // 	st.1.inc r1, r3, r1
        // 	st.1 r1, r4
        // 	add 128, r0, r1
        // 	shl.s 32, r1, r1
        // 	add 256, r1, r1
        // 	shl.s 64, r1, r1
        // 	ret.ok r1
        // "#;

        // let assembly: &'static str = r#"
        // 	.text
        // 	.file	"calldata_array_order.sol:Test"
        // 	.text.unlikely.
        // 	.globl	__entry
        // __entry:
        // 	near_call r0, @inner, @catch_all
        // 	ret.ok r1
        // inner:
        // 	event.first r0, r0
        // 	add r0, r0, r1
        // 	ret.ok.to_label r1, @DEFAULT_FAR_RETURN
        // catch_all:
        // 	ret.panic r0
        // "#;

        // let assembly: &'static str = r#"
        // 	.text
        // 	.file	"calldata_array_order.sol:Test"
        // 	.text.unlikely.
        // 	.globl	__entry
        // __entry:
        // 	near_call r0, @__entry, @panic_handler
        // 	ret.ok r0
        // panic_handler:
        // 	ret.panic r0
        // "#;

        let assembly: &'static str = r#"
		.text
		.file	"calldata_array_order.sol:Test"
		.text.unlikely.
		.globl	__entry
	__entry:
		context.this r2
		far_call r0, r2, @panic_handler
		ret.ok r0
	panic_handler:
		ret.revert r0
	"#;
        set_tracing_mode(VmTracingOptions::ManualVerbose);
        run_inner(&[], VmLaunchOption::Default, assembly);

        // run_for_result_only(assembly);
    }

    #[test]
    fn test_from_compiler() {
        let assembly: &'static str = r#"
           
	.text
	.file	"Test_34"
	.globl	__entry
__entry:
.func_begin0:
	add	r1, r0, r4
	add	@CPI0_0[0], r0, r1
	uma.heap_write	r1, r4, r0
	add	@CPI0_1[0], r0, r1
	uma.heap_write	r1, r2, r0
	and	1, r3, r1
	add	0, r0, r2
	sub!	r1, r2, r1
	jump.ne	@.BB0_4
	jump	@.BB0_5
.BB0_3:
	add	@CPI0_0[0], r0, r1
	uma.heap_read	r1, r0, r1
	add	@CPI0_1[0], r0, r2
	uma.heap_read	r2, r0, r2
	shl.s	32, r2, r2
	add	r1, r2, r1
	ret
.BB0_4:
	near_call	r0, @__constructor, @DEFAULT_UNWIND
	jump	@.BB0_3
.BB0_5:
	near_call	r0, @__selector, @DEFAULT_UNWIND
	jump	@.BB0_3
.func_end0:

__constructor:
.func_begin1:
	nop	stack+=[8]
	add	128, r0, stack-[8]
	add	stack-[8], r0, r2
	add	64, r0, r1
	uma.heap_write	r1, r2, r0
	add	0, r0, r1
	sub!	r1, r1, r1
	jump.ne	@.BB1_4
	jump	@.BB1_5
.BB1_4:
	add	0, r0, r1
	ret.revert.to_label	r1, @DEFAULT_FAR_REVERT
.BB1_5:
	add	0, r0, stack-[7]
	add	stack-[8], r0, r1
	add	r1, r0, stack-[0]
	add	stack-[7], r0, r1
	add	@CPI1_0[0], r0, r2
	uma.heap_read	r2, r0, r2
	add	r2, r0, stack-[1]
	shr.s	5, r1, r2
	add	r2, r0, stack-[2]
	and	31, r1, r3
	add	r3, r0, stack-[3]
	add	@CPI1_1[0], r0, r3
	and	r1, r3, r1
	add	r1, r0, stack-[4]
	add	0, r0, r1
	sub!	r2, r1, r2
	add	r1, r0, stack-[5]
	jump.eq	@.BB1_9
	jump	@.BB1_6
.BB1_6:
	add	stack-[5], 0, r1
	add	stack-[2], 0, r2
	add	stack-[0], 0, r3
	add	stack-[1], 0, r4
	shl.s	5, r1, r5
	add	r4, r5, r4
	uma.calldata_read	r4, r0, r4
	add	r3, r5, r3
	uma.heap_write	r3, r4, r0
	add	1, r1, r1
	sub!	r1, r2, r2
	add	r1, r0, stack-[5]
	jump.lt	@.BB1_6
	jump	@.BB1_9
.BB1_7:
	add	stack-[0], 0, r1
	add	stack-[4], 0, r3
	add	stack-[3], 0, r4
	add	stack-[1], 0, r2
	add	r2, r3, r2
	uma.calldata_read	r2, r0, r2
	shl.s	3, r4, r4
	sub	256, r4, r5
	shr	r2, r5, r2
	shl	r2, r5, r2
	add	r1, r3, r1
	uma.heap_read	r1, r0, r3
	shl	r3, r4, r3
	shr	r3, r4, r3
	or	r2, r3, r2
	uma.heap_write	r1, r2, r0
	jump	@.BB1_8
.BB1_8:
	add	stack-[8], r0, r2
	add	stack-[7], r0, r1
	shl.s	32, r1, r1
	add	r1, r2, r1
	ret.ok.to_label	r1, @DEFAULT_FAR_RETURN
.BB1_9:
	add	stack-[3], 0, r1
	add	0, r0, r2
	sub!	r1, r2, r1
	jump.ne	@.BB1_7
	jump	@.BB1_8
.func_end1:

__selector:
.func_begin2:
	nop	stack+=[8]
	add	128, r0, stack-[8]
	add	stack-[8], r0, r2
	add	64, r0, r1
	uma.heap_write	r1, r2, r0
	add	@CPI2_0[0], r0, r1
	uma.heap_read	r1, r0, r1
	add	4, r0, r2
	sub!	r1, r2, r1
	add	0, r0, r1
	add.lt	1, r0, r1
	and	1, r1, r1
	add	0, r0, r2
	sub!	r1, r2, r1
	add	0, r0, r1
	add.eq	1, r0, r1
	and	1, r1, r1
	sub!	r1, r2, r1
	jump.ne	@.BB2_4
	jump	@.BB2_5
.BB2_4:
	add	0, r0, stack-[7]
	jump	@.BB2_7
.BB2_5:
	add	0, r0, r1
	ret.revert.to_label	r1, @DEFAULT_FAR_REVERT
.BB2_6:
	jump	@.BB2_5
.BB2_7:
	add	stack-[7], r0, r1
	add	@CPI2_1[0], r0, r2
	uma.heap_read	r2, r0, r2
	add	r1, r2, r1
	uma.calldata_read	r1, r0, r1
	add	r1, r0, stack-[1]
	add	0, r0, r1
	sub!	r1, r1, r1
	jump.ne	@.BB2_13
	jump	@.BB2_14
.BB2_8:
	add	0, r0, r2
	add	1, r0, r1
	sub!	r1, r2, r1
	jump.ne	@.BB2_10
	jump	@.BB2_9
.BB2_9:
	add	stack-[7], r0, r2
	shl.s	32, r2, r1
	add	r1, r2, r1
	ret.revert.to_label	r1, @DEFAULT_FAR_REVERT
.BB2_10:
	add	@CPI2_0[0], r0, r1
	uma.heap_read	r1, r0, r1
	near_call	r0, @abi_decode_bool, @DEFAULT_UNWIND
	add	0, r0, r2
	sub!	r1, r2, r1
	add	0, r0, r1
	add.eq	1, r0, r1
	and	1, r1, r1
	sub!	r1, r2, r1
	add	0, r0, r1
	add.eq	1, r0, r1
	and	1, r1, r1
	sub!	r1, r2, r1
	jump.eq	@.BB2_12
	jump	@.BB2_11
.BB2_11:
	add	stack-[7], r0, r2
	shl.s	32, r2, r1
	add	r1, r2, r1
	ret.revert.to_label	r1, @DEFAULT_FAR_REVERT
.BB2_12:
	add	stack-[8], r0, r2
	add	stack-[7], r0, r1
	shl.s	32, r1, r1
	add	r1, r2, r1
	ret.ok.to_label	r1, @DEFAULT_FAR_RETURN
.BB2_13:
	add	0, r0, stack-[6]
	jump	@.BB2_15
.BB2_14:
	add	stack-[1], 0, r1
	shr.s	224, r1, stack-[6]
	jump	@.BB2_15
.BB2_15:
	add	stack-[6], r0, r2
	add	@CPI2_2[0], r0, r1
	sub!	r1, r2, r1
	jump.eq	@.BB2_8
	jump	@.BB2_16
.BB2_16:
	add	stack-[7], r0, r1
	add	@CPI2_1[0], r0, r2
	uma.heap_read	r2, r0, r2
	add	r1, r2, r1
	uma.calldata_read	r1, r0, r1
	add	r1, r0, stack-[0]
	add	0, r0, r1
	sub!	r1, r1, r1
	jump.ne	@.BB2_20
	jump	@.BB2_21
.BB2_17:
	add	0, r0, r2
	add	1, r0, r1
	sub!	r1, r2, r1
	jump.ne	@.BB2_19
	jump	@.BB2_18
.BB2_18:
	add	stack-[7], r0, r2
	shl.s	32, r2, r1
	add	r1, r2, r1
	ret.revert.to_label	r1, @DEFAULT_FAR_REVERT
.BB2_19:
	add	@CPI2_0[0], r0, r1
	uma.heap_read	r1, r0, r1
	near_call	r0, @abi_decode_bool, @DEFAULT_UNWIND
	near_call	r0, @fun_entry, @DEFAULT_UNWIND
	add	r1, r0, stack-[4]
	add	64, r0, r1
	uma.heap_read	r1, r0, r1
	add	r1, r0, stack-[3]
	add	stack-[3], r0, r1
	add	stack-[4], r0, r2
	uma.heap_write	r1, r2, r0
	add	stack-[3], r0, r1
	add	@CPI2_4[0], r0, r2
	add	r1, r2, r1
	ret.ok.to_label	r1, @DEFAULT_FAR_RETURN
.BB2_20:
	add	0, r0, stack-[5]
	jump	@.BB2_22
.BB2_21:
	add	stack-[0], 0, r1
	shr.s	224, r1, stack-[5]
	jump	@.BB2_22
.BB2_22:
	add	stack-[5], r0, r2
	add	@CPI2_3[0], r0, r1
	sub!	r1, r2, r1
	jump.eq	@.BB2_17
	jump	@.BB2_23
.BB2_23:
	jump	@.BB2_6
.func_end2:

abi_decode_bool:
.func_begin3:
	nop	stack+=[3]
	add	0, r0, stack-[3]
	add	r1, r0, stack-[2]
	add	stack-[2], r0, r1
	add	@CPI3_0[0], r0, r2
	add	r1, r2, r2
	add	32, r0, r1
	add	@CPI3_1[0], r0, r5
	sub!	r2, r1, r1
	add	0, r0, r1
	add.lt	r5, r0, r1
	and	r2, r5, r4
	add	0, r0, r2
	sub!	r4, r2, r3
	add	0, r0, r3
	add.gt	r5, r0, r3
	sub!	r4, r5, r4
	add	r1, r0, r1
	add.eq	r3, r0, r1
	sub!	r1, r2, r1
	add	0, r0, r1
	add.ne	1, r0, r1
	and	1, r1, r1
	sub!	r1, r2, r1
	jump.ne	@.BB3_4
	jump	@.BB3_5
.BB3_3:
	add	stack-[3], r0, r1
	nop	stack-=[3]
	ret
.BB3_4:
	add	0, r0, r1
	ret.revert.to_label	r1, @DEFAULT_FAR_REVERT
.BB3_5:
	add	@CPI3_2[0], r0, r1
	uma.heap_read	r1, r0, r1
	add	4, r1, r1
	uma.calldata_read	r1, r0, r1
	add	r1, r0, stack-[1]
	add	stack-[1], r0, r1
	add	0, r0, r2
	sub!	r1, r2, r3
	add	0, r0, r3
	add.eq	1, r0, r3
	and	1, r3, r3
	sub!	r3, r2, r3
	add	0, r0, r3
	add.eq	1, r0, r3
	and	1, r3, r3
	sub!	r1, r3, r1
	add	0, r0, r1
	add.eq	1, r0, r1
	and	1, r1, r1
	sub!	r1, r2, r1
	add	0, r0, r1
	add.eq	1, r0, r1
	and	1, r1, r1
	sub!	r1, r2, r1
	jump.eq	@.BB3_7
	jump	@.BB3_6
.BB3_6:
	add	0, r0, r1
	ret.revert.to_label	r1, @DEFAULT_FAR_REVERT
.BB3_7:
	add	stack-[1], r0, r1
	add	r1, r0, stack-[3]
	jump	@.BB3_3
.func_end3:

fun_entry:
.func_begin4:
	nop	stack+=[15]
	add	0, r0, stack-[15]
	add	r1, r0, stack-[14]
	add	0, r0, r1
	sub!	r1, r1, r1
	jump.ne	@.BB4_4
	jump	@.BB4_5
.BB4_2:
.tmp2:
	add	0, r0, r3
	add	r3, r0, r1
	add	r3, r0, r2
	revert
.BB4_3:
	add	stack-[15], r0, r1
	nop	stack-=[15]
	ret
.BB4_4:
	add	0, r0, r1
	ret.revert.to_label	r1, @DEFAULT_FAR_REVERT
.BB4_5:
	add	64, r0, r1
	uma.heap_read	r1, r0, r1
	add	r1, r0, stack-[13]
	add	stack-[13], r0, r1
	add	r1, r0, stack-[4]
	add	0, r0, r2
	add	1, r0, r1
	sub!	r1, r2, r1
	jump.ne	@.BB4_7
	jump	@.BB4_6
.BB4_6:
	add	0, r0, stack-[12]
	jump	@.BB4_8
.BB4_7:
	add	@CPI4_0[0], r0, r1
	add	r1, r0, stack-[12]
	jump	@.BB4_8
.BB4_8:
	add	stack-[4], 0, r1
	add	stack-[12], r0, r2
	uma.heap_write	r1, r2, r0
	add	stack-[13], r0, r1
	add	4, r1, r1
	add	stack-[14], r0, r2
	add	0, r0, r3
	sub!	r2, r3, r2
	add	0, r0, r2
	add.eq	1, r0, r2
	and	1, r2, r2
	sub!	r2, r3, r2
	add	0, r0, r2
	add.eq	1, r0, r2
	and	1, r2, r2
	uma.heap_write	r1, r2, r0
	context.ergs_left	r1
	add	r1, r0, stack-[1]
	context.this	r1
	add	r1, r0, stack-[2]
	add	stack-[13], r0, r2
	add	r2, r0, stack-[3]
	add	r1, r0, stack-[10]
	add	0, r0, stack-[9]
	jump	@.BB4_9
.BB4_9:
	add	stack-[2], 0, r2
	add	65532, r0, r1
	add	@CPI4_1[0], r0, r5
	sub!	r2, r1, r1
	add	0, r0, r1
	add.lt	r5, r0, r1
	and	r2, r5, r4
	add	0, r0, r2
	sub!	r4, r2, r3
	add	0, r0, r3
	add.gt	r5, r0, r3
	sub!	r4, r5, r4
	add	r1, r0, r1
	add.eq	r3, r0, r1
	sub!	r1, r2, r1
	jump.ne	@.BB4_14
	jump	@.BB4_10
.BB4_10:
	add	stack-[2], 0, r2
	add	65534, r0, r1
	add	@CPI4_1[0], r0, r5
	sub!	r2, r1, r1
	add	0, r0, r1
	add.lt	r5, r0, r1
	and	r2, r5, r4
	add	0, r0, r2
	sub!	r4, r2, r3
	add	0, r0, r3
	add.gt	r5, r0, r3
	sub!	r4, r5, r4
	add	r1, r0, r1
	add.eq	r3, r0, r1
	sub!	r1, r2, r1
	jump.ne	@.BB4_13
	jump	@.BB4_11
.BB4_11:
	add	stack-[2], 0, r2
	add	65535, r0, r1
	add	@CPI4_1[0], r0, r5
	sub!	r2, r1, r1
	add	0, r0, r1
	add.lt	r5, r0, r1
	and	r2, r5, r4
	add	0, r0, r2
	sub!	r4, r2, r3
	add	0, r0, r3
	add.gt	r5, r0, r3
	sub!	r4, r5, r4
	add	r1, r0, r1
	add.eq	r3, r0, r1
	sub!	r1, r2, r1
	jump.ne	@.BB4_24
	jump	@.BB4_12
.BB4_12:
	add	stack-[2], 0, r1
	add	65535, r0, r2
	sub!	r1, r2, r1
	jump.eq	@.BB4_22
	jump	@.BB4_26
.BB4_13:
	add	stack-[2], 0, r2
	add	65533, r0, r1
	add	@CPI4_1[0], r0, r5
	sub!	r2, r1, r1
	add	0, r0, r1
	add.lt	r5, r0, r1
	and	r2, r5, r4
	add	0, r0, r2
	sub!	r4, r2, r3
	add	0, r0, r3
	add.gt	r5, r0, r3
	sub!	r4, r5, r4
	add	r1, r0, r1
	add.eq	r3, r0, r1
	sub!	r1, r2, r1
	jump.ne	@.BB4_25
	jump	@.BB4_23
.BB4_14:
	add	stack-[2], 0, r2
	add	2, r0, r1
	add	@CPI4_1[0], r0, r5
	sub!	r2, r1, r1
	add	0, r0, r1
	add.lt	r5, r0, r1
	and	r2, r5, r4
	add	0, r0, r2
	sub!	r4, r2, r3
	add	0, r0, r3
	add.gt	r5, r0, r3
	sub!	r4, r5, r4
	add	r1, r0, r1
	add.eq	r3, r0, r1
	sub!	r1, r2, r1
	jump.ne	@.BB4_18
	jump	@.BB4_15
.BB4_15:
	add	stack-[2], 0, r2
	add	4, r0, r1
	add	@CPI4_1[0], r0, r5
	sub!	r2, r1, r1
	add	0, r0, r1
	add.lt	r5, r0, r1
	and	r2, r5, r4
	add	0, r0, r2
	sub!	r4, r2, r3
	add	0, r0, r3
	add.gt	r5, r0, r3
	sub!	r4, r5, r4
	add	r1, r0, r1
	add.eq	r3, r0, r1
	sub!	r1, r2, r1
	jump.ne	@.BB4_17
	jump	@.BB4_16
.BB4_16:
	add	stack-[2], 0, r1
	add	4, r0, r2
	sub!	r1, r2, r1
	jump.eq	@.BB4_21
	jump	@.BB4_26
.BB4_17:
	add	stack-[2], 0, r1
	add	2, r0, r2
	sub!	r1, r2, r1
	jump.eq	@.BB4_20
	jump	@.BB4_26
.BB4_18:
	add	stack-[2], 0, r1
	add	1, r0, r2
	sub!	r1, r2, r1
	jump.ne	@.BB4_26
	jump	@.BB4_19
.BB4_19:
	add	18, r0, stack-[10]
	jump	@.BB4_27
.BB4_20:
	add	17, r0, stack-[10]
	jump	@.BB4_27
.BB4_21:
	add	1, r0, stack-[9]
	jump	@.BB4_28
.BB4_22:
	add	stack-[3], 0, r2
	add	0, r0, r1
	to_l1.first	r1, r2
	jump	@.BB4_28
.BB4_23:
	add	stack-[1], 0, r1
	add	stack-[3], 0, r2
	precompile	r1, r2, r1
	add	r1, r0, stack-[9]
	jump	@.BB4_28
.BB4_24:
	context.code_source	r1
	add	r1, r0, stack-[9]
	jump	@.BB4_28
.BB4_25:
	context.meta	r1
	add	r1, r0, stack-[9]
	jump	@.BB4_28
.BB4_26:
	jump	@.BB4_27
.BB4_27:
	add	stack-[3], 0, r1
	add	stack-[10], r0, r2
	add	@CPI4_2[0], r0, r3
	add	r1, r3, r1
.tmp0:
	context.sp	r3
	sub.s	7, r3, r3
	mul	32, r3, r3, r0
	near_call	r0, @__staticcall, @.BB4_2
.tmp1:
	add	r1, r0, stack-[0]
	jump	@.BB4_29
.BB4_28:
	add	stack-[9], r0, r1
	add	r1, r0, stack-[11]
	add	stack-[11], r0, r1
	add	0, r0, r2
	sub!	r1, r2, r1
	jump.ne	@.BB4_30
	jump	@.BB4_31
.BB4_29:
	add	stack-[0], 0, r1
	div.s	32, r1, r1, r0
	add	stack[r1 - 1], r0, r1
	add	r1, r0, stack-[9]
	jump	@.BB4_28
.BB4_30:
	add	stack-[13], r0, r1
	add	@CPI4_3[0], r0, r2
	sub!	r1, r2, r1
	add	0, r0, r1
	add.gt	1, r0, r1
	and	1, r1, r1
	add	0, r0, r2
	sub!	r1, r2, r1
	jump.ne	@.BB4_32
	jump	@.BB4_33
.BB4_31:
	jump	@.BB4_38
.BB4_32:
	add	0, r0, r1
	sub!	r1, r1, r1
	jump.ne	@.BB4_34
	jump	@.BB4_35
.BB4_33:
	add	stack-[13], r0, r2
	add	64, r0, r1
	uma.heap_write	r1, r2, r0
	jump	@.BB4_31
.BB4_34:
	add	0, r0, stack-[6]
	jump	@.BB4_36
.BB4_35:
	add	@CPI4_4[0], r0, r1
	add	r1, r0, stack-[6]
	jump	@.BB4_36
.BB4_36:
	add	stack-[6], r0, r2
	add	0, r0, r1
	uma.heap_write	r1, r2, r0
	add	65, r0, r2
	add	4, r0, r1
	uma.heap_write	r1, r2, r0
	add	@CPI4_2[0], r0, r1
	ret.revert.to_label	r1, @DEFAULT_FAR_REVERT
.BB4_38:
	add	stack-[11], r0, r2
	add	0, r0, r1
	sub!	r2, r1, r2
	add	0, r0, r2
	add.eq	1, r0, r2
	and	1, r2, r2
	sub!	r1, r2, r1
	jump.ne	@.BB4_40
	jump	@.BB4_39
.BB4_39:
	add	1, r0, stack-[15]
	jump	@.BB4_3
.BB4_40:
	add	2, r0, stack-[15]
	jump	@.BB4_3
.func_end4:

__cxa_throw:
	revert

__staticcall:
.func_begin5:
	nop	stack+=[5]
	add	r3, r0, r4
	add	r4, r0, stack-[1]
	add	r2, r0, r3
	add	r1, r0, r2
	add	r4, r0, r5
	add	r5, r0, stack-[2]
	add	32, r4, r4
	add	r4, r0, stack-[3]
.tmp3:
	far_call.static	r3, r2, @.BB6_2
.tmp4:
	add	r1, r0, stack-[4]
	jump	@.BB6_1
.BB6_1:
	add	stack-[1], 0, r1
	add	stack-[3], 0, r2
	add	stack-[4], 0, r3
	add	stack-[2], 0, r4
	div.s	32, r4, r4, r0
	add	r3, r0, stack[r4 - 1]
	div.s	32, r2, r3, r4
	sub	r2, r4, r2
	div.s	32, r2, r3, r0
	add	stack[r3 - 2], r0, r3
	shl.s	3, r4, r4
	shl	r3, r4, r5
	shr	r5, r4, r7
	sub	256, r4, r6
	shl	1, r6, r5
	or	r5, r7, r8
	add	0, r0, r5
	sub!	r4, r5, r7
	div.s	32, r2, r7, r0
	add	r8, r0, stack[r7 - 2]
	add.eq	r3, r0, stack[r7 - 2]
	div.s	32, r2, r3, r0
	add	stack[r3 - 1], r0, r3
	shr	r3, r6, r3
	shl	r3, r6, r6
	shr	1, r4, r3
	or	r3, r6, r3
	sub!	r4, r5, r4
	div.s	32, r2, r2, r0
	add	r3, r0, stack[r2 - 1]
	add.eq	1, r0, stack[r2 - 1]
	nop	stack-=[5]
	ret
.BB6_2:
.tmp5:
	add	r1, r0, stack-[0]
	add	0, r0, r1
	add.lt	1, r0, r1
	and	1, r1, r1
	add	1, r0, r2
	sub!	r1, r2, r1
	jump.ne	@.BB6_4
	jump	@.BB6_3
.BB6_3:
	add	stack-[0], 0, r1
	revert
.BB6_4:
	add	stack-[1], 0, r1
	add	stack-[3], 0, r2
	add	stack-[0], 0, r3
	add	stack-[2], 0, r4
	div.s	32, r4, r4, r0
	add	r3, r0, stack[r4 - 1]
	div.s	32, r2, r3, r4
	sub	r2, r4, r2
	div.s	32, r2, r3, r0
	add	stack[r3 - 2], r0, r3
	shl.s	3, r4, r4
	shl	r3, r4, r5
	shr	r5, r4, r7
	add	0, r0, r5
	sub!	r4, r5, r6
	div.s	32, r2, r6, r0
	add	r7, r0, stack[r6 - 2]
	add.eq	r3, r0, stack[r6 - 2]
	div.s	32, r2, r3, r0
	add	stack[r3 - 1], r0, r3
	sub	256, r4, r6
	shr	r3, r6, r3
	shl	r3, r6, r3
	sub!	r4, r5, r4
	div.s	32, r2, r2, r0
	add	r3, r0, stack[r2 - 1]
	add.eq	0, r0, stack[r2 - 1]
	nop	stack-=[5]
	ret
.func_end5:

	.note.GNU-stack
	.rodata
CPI0_0:
	.cell 16777184
CPI0_1:
	.cell 16777152
CPI1_0:
	.cell 16777184
CPI1_1:
	.cell -32
CPI2_0:
	.cell 16777152
CPI2_1:
	.cell 16777184
CPI2_2:
	.cell 2562959041
CPI2_3:
	.cell 3509246214
CPI2_4:
	.cell 137438953472
CPI3_0:
	.cell -4
CPI3_1:
	.cell -57896044618658097711785492504343953926634992332820282019728792003956564819968
CPI3_2:
	.cell 16777184
CPI4_0:
	.cell -46694850181864645452894775106927543673790745014467479609697284461944719278080
CPI4_1:
	.cell -57896044618658097711785492504343953926634992332820282019728792003956564819968
CPI4_2:
	.cell 154618822656
CPI4_3:
	.cell 18446744073709551615
CPI4_4:
	.cell 35408467139433450592217433187231851964531694900788300625387963629091585785856
            "#;

        set_tracing_mode(VmTracingOptions::ManualVerbose);
        run_inner(
            &hex::decode(
                "d12ad9060000000000000000000000000000000000000000000000000000000000000001",
            )
            .unwrap(),
            VmLaunchOption::Default,
            assembly,
        );
    }
}
