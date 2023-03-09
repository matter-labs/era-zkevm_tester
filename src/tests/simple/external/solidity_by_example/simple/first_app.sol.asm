	.text
	.file	"Test_28"
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
.BB0_2:
.tmp4:
	add	@CPI0_2[0], r0, r1
	uma.heap_read	r1, r0, r1
	add	1, r0, r2
	sub!	r1, r2, r1
	jump.ne	@.BB0_9
	jump	@.BB0_3
.BB0_3:
	add	@CPI0_0[0], r0, r1
	uma.heap_read	r1, r0, r1
	add	@CPI0_1[0], r0, r2
	uma.heap_read	r2, r0, r2
	shl.s	32, r2, r2
	add	r1, r2, r1
	ret
.BB0_4:
.tmp2:
	near_call	r0, @__constructor, @.BB0_2
.tmp3:
	jump	@.BB0_6
.BB0_5:
.tmp0:
	near_call	r0, @__selector, @.BB0_2
.tmp1:
	jump	@.BB0_7
.BB0_6:
	jump	@.BB0_3
.BB0_7:
	jump	@.BB0_3
.BB0_9:
	add	@CPI0_0[0], r0, r1
	uma.heap_read	r1, r0, r1
	add	@CPI0_1[0], r0, r2
	uma.heap_read	r2, r0, r2
	shl.s	32, r2, r2
	add	r1, r2, r1
	add	0, r0, r3
	add	r3, r0, r2
	near_call	r0, @__cxa_throw, @DEFAULT_UNWIND
.func_end0:

__constructor:
.func_begin1:
	nop	stack+=[2]
	add	128, r0, r2
	add	64, r0, r1
	uma.heap_write	r1, r2, r0
	add	0, r0, r1
	sub!	r1, r1, r1
	jump.ne	@.BB1_4
	jump	@.BB1_5
.BB1_2:
.tmp11:
	add	0, r0, r3
	add	r3, r0, r1
	add	r3, r0, r2
	near_call	r0, @__cxa_throw, @DEFAULT_UNWIND
.BB1_3:
	nop	stack-=[2]
	ret
.BB1_4:
.tmp5:
	near_call	r0, @revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb, @.BB1_2
.tmp6:
	jump	@.BB1_6
.BB1_5:
.tmp7:
	near_call	r0, @constructor_Test_28, @.BB1_2
.tmp8:
	jump	@.BB1_7
.BB1_6:
	jump	@.BB1_5
.BB1_7:
.tmp9:
	near_call	r0, @allocate_unbounded, @.BB1_2
.tmp10:
	add	r1, r0, stack-[0]
	jump	@.BB1_8
.BB1_8:
	add	stack-[0], 0, r1
	add	r1, r0, stack-[2]
	add	stack-[2], r0, r2
	add	@CPI1_0[0], r0, r1
	uma.heap_write	r1, r2, r0
	add	0, r0, r2
	add	@CPI1_1[0], r0, r1
	uma.heap_write	r1, r2, r0
	jump	@.BB1_3
.func_end1:

__selector:
.func_begin2:
	nop	stack+=[2]
	add	128, r0, r2
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
.BB2_2:
.tmp24:
	add	0, r0, r3
	add	r3, r0, r1
	add	r3, r0, r2
	near_call	r0, @__cxa_throw, @DEFAULT_UNWIND
.BB2_3:
	nop	stack-=[2]
	ret
.BB2_4:
	add	@CPI2_1[0], r0, r1
	uma.heap_read	r1, r0, r1
	uma.calldata_read	r1, r0, r1
.tmp12:
	near_call	r0, @shift_right_224_unsigned, @.BB2_2
.tmp13:
	add	r1, r0, stack-[0]
	jump	@.BB2_6
.BB2_5:
.tmp22:
	near_call	r0, @revert_error_42b3090547df1d2001c96683413b8cf91c1b902ef5e3cb8d9f6f304cf7446f74, @.BB2_2
.tmp23:
	jump	@.BB2_21
.BB2_6:
	add	stack-[0], 0, r1
	add	r1, r0, stack-[2]
	jump	@.BB2_8
.BB2_7:
	jump	@.BB2_5
.BB2_8:
	add	stack-[2], r0, r2
	add	@CPI2_2[0], r0, r1
	sub!	r1, r2, r1
	jump.ne	@.BB2_11
	jump	@.BB2_9
.BB2_9:
.tmp20:
	near_call	r0, @external_fun_count_3, @.BB2_2
.tmp21:
	jump	@.BB2_10
.BB2_10:
	jump	@.BB2_7
.BB2_11:
	add	stack-[2], r0, r2
	add	@CPI2_3[0], r0, r1
	sub!	r1, r2, r1
	jump.ne	@.BB2_14
	jump	@.BB2_12
.BB2_12:
.tmp18:
	near_call	r0, @external_fun_inc_19, @.BB2_2
.tmp19:
	jump	@.BB2_13
.BB2_13:
	jump	@.BB2_7
.BB2_14:
	add	stack-[2], r0, r2
	add	@CPI2_4[0], r0, r1
	sub!	r1, r2, r1
	jump.ne	@.BB2_17
	jump	@.BB2_15
.BB2_15:
.tmp16:
	near_call	r0, @external_fun_get_11, @.BB2_2
.tmp17:
	jump	@.BB2_16
.BB2_16:
	jump	@.BB2_7
.BB2_17:
	add	stack-[2], r0, r2
	add	@CPI2_5[0], r0, r1
	sub!	r1, r2, r1
	jump.ne	@.BB2_20
	jump	@.BB2_18
.BB2_18:
.tmp14:
	near_call	r0, @external_fun_dec_27, @.BB2_2
.tmp15:
	jump	@.BB2_19
.BB2_19:
	jump	@.BB2_7
.BB2_20:
	jump	@.BB2_7
.BB2_21:
	jump	@.BB2_3
.func_end2:

allocate_unbounded:
.func_begin3:
	nop	stack+=[1]
	add	0, r0, stack-[1]
	add	64, r0, r1
	uma.heap_read	r1, r0, r1
	add	r1, r0, stack-[1]
	jump	@.BB3_3
.BB3_3:
	add	stack-[1], r0, r1
	nop	stack-=[1]
	ret
.func_end3:

revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb:
.func_begin4:
	add	0, r0, r2
	add	@CPI4_0[0], r0, r1
	uma.heap_write	r1, r2, r0
	add	@CPI4_1[0], r0, r1
	uma.heap_write	r1, r2, r0
	jump	@.BB4_1
.BB4_1:
	add	0, r0, r3
	add	r3, r0, r1
	add	r3, r0, r2
	near_call	r0, @__cxa_throw, @DEFAULT_UNWIND
.func_end4:

constructor_Test_28:
.func_begin5:
	jump	@.BB5_3
.BB5_3:
	ret
.func_end5:

shift_right_224_unsigned:
.func_begin6:
	nop	stack+=[4]
	add	0, r0, stack-[4]
	add	r1, r0, stack-[3]
	add	stack-[3], r0, r1
	add	r1, r0, stack-[0]
	add	0, r0, r1
	sub!	r1, r1, r1
	jump.ne	@.BB6_4
	jump	@.BB6_5
.BB6_3:
	add	stack-[4], r0, r1
	nop	stack-=[4]
	ret
.BB6_4:
	add	0, r0, stack-[2]
	jump	@.BB6_6
.BB6_5:
	add	stack-[0], 0, r1
	shr.s	224, r1, stack-[2]
	jump	@.BB6_6
.BB6_6:
	add	stack-[2], r0, r1
	add	r1, r0, stack-[4]
	jump	@.BB6_3
.func_end6:

allocate_unbounded.1:
.func_begin7:
	nop	stack+=[1]
	add	0, r0, stack-[1]
	add	64, r0, r1
	uma.heap_read	r1, r0, r1
	add	r1, r0, stack-[1]
	jump	@.BB7_3
.BB7_3:
	add	stack-[1], r0, r1
	nop	stack-=[1]
	ret
.func_end7:

revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb.2:
.func_begin8:
	add	0, r0, r2
	add	@CPI8_0[0], r0, r1
	uma.heap_write	r1, r2, r0
	add	@CPI8_1[0], r0, r1
	uma.heap_write	r1, r2, r0
	jump	@.BB8_1
.BB8_1:
	add	0, r0, r3
	add	r3, r0, r1
	add	r3, r0, r2
	near_call	r0, @__cxa_throw, @DEFAULT_UNWIND
.func_end8:

revert_error_dbdddcbe895c83990c08b3492a0e83918d802a52331272ac6fdb6a7c4aea3b1b:
.func_begin9:
	add	0, r0, r2
	add	@CPI9_0[0], r0, r1
	uma.heap_write	r1, r2, r0
	add	@CPI9_1[0], r0, r1
	uma.heap_write	r1, r2, r0
	jump	@.BB9_1
.BB9_1:
	add	0, r0, r3
	add	r3, r0, r1
	add	r3, r0, r2
	near_call	r0, @__cxa_throw, @DEFAULT_UNWIND
.func_end9:

abi_decode_tuple_:
.func_begin10:
	nop	stack+=[2]
	add	r1, r0, stack-[2]
	add	r2, r0, stack-[1]
	add	stack-[1], r0, r1
	add	stack-[2], r0, r2
	sub	r1, r2, r1
	add	@CPI10_0[0], r0, r2
	sub!	r1, r2, r1
	add	0, r0, r1
	add.gt	1, r0, r1
	and	1, r1, r1
	add	0, r0, r2
	sub!	r1, r2, r1
	jump.ne	@.BB10_4
	jump	@.BB10_5
.BB10_2:
.tmp27:
	add	0, r0, r3
	add	r3, r0, r1
	add	r3, r0, r2
	near_call	r0, @__cxa_throw, @DEFAULT_UNWIND
.BB10_3:
	nop	stack-=[2]
	ret
.BB10_4:
.tmp25:
	near_call	r0, @revert_error_dbdddcbe895c83990c08b3492a0e83918d802a52331272ac6fdb6a7c4aea3b1b, @.BB10_2
.tmp26:
	jump	@.BB10_6
.BB10_5:
	jump	@.BB10_3
.BB10_6:
	jump	@.BB10_5
.func_end10:

shift_right_unsigned_dynamic:
.func_begin11:
	nop	stack+=[6]
	add	0, r0, stack-[6]
	add	r1, r0, stack-[5]
	add	r2, r0, stack-[4]
	add	stack-[5], r0, r1
	add	r1, r0, stack-[0]
	add	stack-[4], r0, r2
	add	r2, r0, stack-[1]
	add	255, r0, r2
	sub!	r1, r2, r1
	jump.gt	@.BB11_4
	jump	@.BB11_5
.BB11_3:
	add	stack-[6], r0, r1
	nop	stack-=[6]
	ret
.BB11_4:
	add	0, r0, stack-[3]
	jump	@.BB11_6
.BB11_5:
	add	stack-[1], 0, r1
	add	stack-[0], 0, r2
	shr	r1, r2, stack-[3]
	jump	@.BB11_6
.BB11_6:
	add	stack-[3], r0, r1
	add	r1, r0, stack-[6]
	jump	@.BB11_3
.func_end11:

cleanup_from_storage_t_uint256:
.func_begin12:
	nop	stack+=[2]
	add	0, r0, stack-[2]
	add	r1, r0, stack-[1]
	add	stack-[1], r0, r1
	add	r1, r0, stack-[2]
	jump	@.BB12_3
.BB12_3:
	add	stack-[2], r0, r1
	nop	stack-=[2]
	ret
.func_end12:

extract_from_storage_value_dynamict_uint256:
.func_begin13:
	nop	stack+=[5]
	add	0, r0, stack-[5]
	add	r1, r0, stack-[4]
	add	r2, r0, stack-[3]
	add	stack-[3], r0, r1
	shl.s	3, r1, r1
	add	stack-[4], r0, r2
.tmp28:
	near_call	r0, @shift_right_unsigned_dynamic, @.BB13_2
.tmp29:
	add	r1, r0, stack-[1]
	jump	@.BB13_4
.BB13_2:
.tmp32:
	add	0, r0, r3
	add	r3, r0, r1
	add	r3, r0, r2
	near_call	r0, @__cxa_throw, @DEFAULT_UNWIND
.BB13_3:
	add	stack-[5], r0, r1
	nop	stack-=[5]
	ret
.BB13_4:
.tmp30:
	add	stack-[1], 0, r1
	near_call	r0, @cleanup_from_storage_t_uint256, @.BB13_2
.tmp31:
	add	r1, r0, stack-[0]
	jump	@.BB13_5
.BB13_5:
	add	stack-[0], 0, r1
	add	r1, r0, stack-[5]
	jump	@.BB13_3
.func_end13:

read_from_storage_split_dynamic_t_uint256:
.func_begin14:
	nop	stack+=[5]
	add	0, r0, stack-[5]
	add	r1, r0, stack-[4]
	add	r2, r0, stack-[3]
	add	stack-[4], r0, r1
.tmp33:
	near_call	r0, @__sload, @.BB14_2
.tmp34:
	add	r1, r0, stack-[1]
	jump	@.BB14_4
.BB14_2:
.tmp37:
	add	0, r0, r3
	add	r3, r0, r1
	add	r3, r0, r2
	near_call	r0, @__cxa_throw, @DEFAULT_UNWIND
.BB14_3:
	add	stack-[5], r0, r1
	nop	stack-=[5]
	ret
.BB14_4:
	add	stack-[1], 0, r1
	add	stack-[3], r0, r2
.tmp35:
	near_call	r0, @extract_from_storage_value_dynamict_uint256, @.BB14_2
.tmp36:
	add	r1, r0, stack-[0]
	jump	@.BB14_5
.BB14_5:
	add	stack-[0], 0, r1
	add	r1, r0, stack-[5]
	jump	@.BB14_3
.func_end14:

getter_fun_count_3:
.func_begin15:
	nop	stack+=[4]
	add	0, r0, stack-[4]
	add	0, r0, stack-[3]
	add	0, r0, stack-[2]
	add	stack-[3], r0, r1
	add	stack-[2], r0, r2
.tmp38:
	near_call	r0, @read_from_storage_split_dynamic_t_uint256, @.BB15_2
.tmp39:
	add	r1, r0, stack-[0]
	jump	@.BB15_4
.BB15_2:
.tmp40:
	add	0, r0, r3
	add	r3, r0, r1
	add	r3, r0, r2
	near_call	r0, @__cxa_throw, @DEFAULT_UNWIND
.BB15_3:
	add	stack-[4], r0, r1
	nop	stack-=[4]
	ret
.BB15_4:
	add	stack-[0], 0, r1
	add	r1, r0, stack-[4]
	jump	@.BB15_3
.func_end15:

cleanup_t_uint256:
.func_begin16:
	nop	stack+=[2]
	add	0, r0, stack-[2]
	add	r1, r0, stack-[1]
	add	stack-[1], r0, r1
	add	r1, r0, stack-[2]
	jump	@.BB16_3
.BB16_3:
	add	stack-[2], r0, r1
	nop	stack-=[2]
	ret
.func_end16:

abi_encode_t_uint256_to_t_uint256_fromStack:
.func_begin17:
	nop	stack+=[4]
	add	r1, r0, stack-[4]
	add	r2, r0, stack-[3]
	add	stack-[3], r0, r1
	add	r1, r0, stack-[0]
	add	stack-[4], r0, r1
.tmp41:
	near_call	r0, @cleanup_t_uint256, @.BB17_2
.tmp42:
	add	r1, r0, stack-[1]
	jump	@.BB17_4
.BB17_2:
.tmp43:
	add	0, r0, r3
	add	r3, r0, r1
	add	r3, r0, r2
	near_call	r0, @__cxa_throw, @DEFAULT_UNWIND
.BB17_3:
	nop	stack-=[4]
	ret
.BB17_4:
	add	stack-[0], 0, r1
	add	stack-[1], 0, r2
	uma.heap_write	r1, r2, r0
	jump	@.BB17_3
.func_end17:

abi_encode_tuple_t_uint256__to_t_uint256__fromStack:
.func_begin18:
	nop	stack+=[3]
	add	0, r0, stack-[3]
	add	r1, r0, stack-[2]
	add	r2, r0, stack-[1]
	add	stack-[2], r0, r1
	add	32, r1, stack-[3]
	add	stack-[1], r0, r1
	add	stack-[2], r0, r2
.tmp44:
	near_call	r0, @abi_encode_t_uint256_to_t_uint256_fromStack, @.BB18_2
.tmp45:
	jump	@.BB18_4
.BB18_2:
.tmp46:
	add	0, r0, r3
	add	r3, r0, r1
	add	r3, r0, r2
	near_call	r0, @__cxa_throw, @DEFAULT_UNWIND
.BB18_3:
	add	stack-[3], r0, r1
	nop	stack-=[3]
	ret
.BB18_4:
	jump	@.BB18_3
.func_end18:

external_fun_count_3:
.func_begin19:
	nop	stack+=[6]
	add	0, r0, r1
	sub!	r1, r1, r1
	jump.ne	@.BB19_4
	jump	@.BB19_5
.BB19_1:
	add	0, r0, r3
	add	r3, r0, r1
	add	r3, r0, r2
	near_call	r0, @__cxa_throw, @DEFAULT_UNWIND
.BB19_2:
.tmp57:
	add	0, r0, r3
	add	r3, r0, r1
	add	r3, r0, r2
	near_call	r0, @__cxa_throw, @DEFAULT_UNWIND
.BB19_4:
.tmp47:
	near_call	r0, @revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb.2, @.BB19_2
.tmp48:
	jump	@.BB19_6
.BB19_5:
	add	@CPI19_0[0], r0, r1
	uma.heap_read	r1, r0, r2
.tmp49:
	add	4, r0, r1
	near_call	r0, @abi_decode_tuple_, @.BB19_2
.tmp50:
	jump	@.BB19_7
.BB19_6:
	jump	@.BB19_5
.BB19_7:
.tmp51:
	near_call	r0, @getter_fun_count_3, @.BB19_2
.tmp52:
	add	r1, r0, stack-[2]
	jump	@.BB19_8
.BB19_8:
	add	stack-[2], 0, r1
	add	r1, r0, stack-[6]
.tmp53:
	near_call	r0, @allocate_unbounded.1, @.BB19_2
.tmp54:
	add	r1, r0, stack-[1]
	jump	@.BB19_9
.BB19_9:
	add	stack-[1], 0, r1
	add	r1, r0, stack-[5]
	add	stack-[5], r0, r1
	add	stack-[6], r0, r2
.tmp55:
	near_call	r0, @abi_encode_tuple_t_uint256__to_t_uint256__fromStack, @.BB19_2
.tmp56:
	add	r1, r0, stack-[0]
	jump	@.BB19_10
.BB19_10:
	add	stack-[0], 0, r1
	add	r1, r0, stack-[4]
	add	stack-[5], r0, r3
	add	stack-[4], r0, r1
	sub	r1, r3, r2
	add	@CPI19_1[0], r0, r1
	uma.heap_write	r1, r3, r0
	add	@CPI19_0[0], r0, r1
	uma.heap_write	r1, r2, r0
	add	1, r0, r2
	add	@CPI19_2[0], r0, r1
	uma.heap_write	r1, r2, r0
	jump	@.BB19_1
.func_end19:

abi_encode_tuple__to__fromStack:
.func_begin20:
	nop	stack+=[2]
	add	0, r0, stack-[2]
	add	r1, r0, stack-[1]
	add	stack-[1], r0, r1
	add	r1, r0, stack-[2]
	jump	@.BB20_3
.BB20_3:
	add	stack-[2], r0, r1
	nop	stack-=[2]
	ret
.func_end20:

external_fun_inc_19:
.func_begin21:
	nop	stack+=[4]
	add	0, r0, r1
	sub!	r1, r1, r1
	jump.ne	@.BB21_4
	jump	@.BB21_5
.BB21_1:
	add	0, r0, r3
	add	r3, r0, r1
	add	r3, r0, r2
	near_call	r0, @__cxa_throw, @DEFAULT_UNWIND
.BB21_2:
.tmp68:
	add	0, r0, r3
	add	r3, r0, r1
	add	r3, r0, r2
	near_call	r0, @__cxa_throw, @DEFAULT_UNWIND
.BB21_4:
.tmp58:
	near_call	r0, @revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb.2, @.BB21_2
.tmp59:
	jump	@.BB21_6
.BB21_5:
	add	@CPI21_0[0], r0, r1
	uma.heap_read	r1, r0, r2
.tmp60:
	add	4, r0, r1
	near_call	r0, @abi_decode_tuple_, @.BB21_2
.tmp61:
	jump	@.BB21_7
.BB21_6:
	jump	@.BB21_5
.BB21_7:
.tmp62:
	near_call	r0, @fun_inc_19, @.BB21_2
.tmp63:
	jump	@.BB21_8
.BB21_8:
.tmp64:
	near_call	r0, @allocate_unbounded.1, @.BB21_2
.tmp65:
	add	r1, r0, stack-[1]
	jump	@.BB21_9
.BB21_9:
	add	stack-[1], 0, r1
	add	r1, r0, stack-[4]
	add	stack-[4], r0, r1
.tmp66:
	near_call	r0, @abi_encode_tuple__to__fromStack, @.BB21_2
.tmp67:
	add	r1, r0, stack-[0]
	jump	@.BB21_10
.BB21_10:
	add	stack-[0], 0, r1
	add	r1, r0, stack-[3]
	add	stack-[4], r0, r3
	add	stack-[3], r0, r1
	sub	r1, r3, r2
	add	@CPI21_1[0], r0, r1
	uma.heap_write	r1, r3, r0
	add	@CPI21_0[0], r0, r1
	uma.heap_write	r1, r2, r0
	add	1, r0, r2
	add	@CPI21_2[0], r0, r1
	uma.heap_write	r1, r2, r0
	jump	@.BB21_1
.func_end21:

external_fun_get_11:
.func_begin22:
	nop	stack+=[6]
	add	0, r0, r1
	sub!	r1, r1, r1
	jump.ne	@.BB22_4
	jump	@.BB22_5
.BB22_1:
	add	0, r0, r3
	add	r3, r0, r1
	add	r3, r0, r2
	near_call	r0, @__cxa_throw, @DEFAULT_UNWIND
.BB22_2:
.tmp79:
	add	0, r0, r3
	add	r3, r0, r1
	add	r3, r0, r2
	near_call	r0, @__cxa_throw, @DEFAULT_UNWIND
.BB22_4:
.tmp69:
	near_call	r0, @revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb.2, @.BB22_2
.tmp70:
	jump	@.BB22_6
.BB22_5:
	add	@CPI22_0[0], r0, r1
	uma.heap_read	r1, r0, r2
.tmp71:
	add	4, r0, r1
	near_call	r0, @abi_decode_tuple_, @.BB22_2
.tmp72:
	jump	@.BB22_7
.BB22_6:
	jump	@.BB22_5
.BB22_7:
.tmp73:
	near_call	r0, @fun_get_11, @.BB22_2
.tmp74:
	add	r1, r0, stack-[2]
	jump	@.BB22_8
.BB22_8:
	add	stack-[2], 0, r1
	add	r1, r0, stack-[6]
.tmp75:
	near_call	r0, @allocate_unbounded.1, @.BB22_2
.tmp76:
	add	r1, r0, stack-[1]
	jump	@.BB22_9
.BB22_9:
	add	stack-[1], 0, r1
	add	r1, r0, stack-[5]
	add	stack-[5], r0, r1
	add	stack-[6], r0, r2
.tmp77:
	near_call	r0, @abi_encode_tuple_t_uint256__to_t_uint256__fromStack, @.BB22_2
.tmp78:
	add	r1, r0, stack-[0]
	jump	@.BB22_10
.BB22_10:
	add	stack-[0], 0, r1
	add	r1, r0, stack-[4]
	add	stack-[5], r0, r3
	add	stack-[4], r0, r1
	sub	r1, r3, r2
	add	@CPI22_1[0], r0, r1
	uma.heap_write	r1, r3, r0
	add	@CPI22_0[0], r0, r1
	uma.heap_write	r1, r2, r0
	add	1, r0, r2
	add	@CPI22_2[0], r0, r1
	uma.heap_write	r1, r2, r0
	jump	@.BB22_1
.func_end22:

external_fun_dec_27:
.func_begin23:
	nop	stack+=[4]
	add	0, r0, r1
	sub!	r1, r1, r1
	jump.ne	@.BB23_4
	jump	@.BB23_5
.BB23_1:
	add	0, r0, r3
	add	r3, r0, r1
	add	r3, r0, r2
	near_call	r0, @__cxa_throw, @DEFAULT_UNWIND
.BB23_2:
.tmp90:
	add	0, r0, r3
	add	r3, r0, r1
	add	r3, r0, r2
	near_call	r0, @__cxa_throw, @DEFAULT_UNWIND
.BB23_4:
.tmp80:
	near_call	r0, @revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb.2, @.BB23_2
.tmp81:
	jump	@.BB23_6
.BB23_5:
	add	@CPI23_0[0], r0, r1
	uma.heap_read	r1, r0, r2
.tmp82:
	add	4, r0, r1
	near_call	r0, @abi_decode_tuple_, @.BB23_2
.tmp83:
	jump	@.BB23_7
.BB23_6:
	jump	@.BB23_5
.BB23_7:
.tmp84:
	near_call	r0, @fun_dec_27, @.BB23_2
.tmp85:
	jump	@.BB23_8
.BB23_8:
.tmp86:
	near_call	r0, @allocate_unbounded.1, @.BB23_2
.tmp87:
	add	r1, r0, stack-[1]
	jump	@.BB23_9
.BB23_9:
	add	stack-[1], 0, r1
	add	r1, r0, stack-[4]
	add	stack-[4], r0, r1
.tmp88:
	near_call	r0, @abi_encode_tuple__to__fromStack, @.BB23_2
.tmp89:
	add	r1, r0, stack-[0]
	jump	@.BB23_10
.BB23_10:
	add	stack-[0], 0, r1
	add	r1, r0, stack-[3]
	add	stack-[4], r0, r3
	add	stack-[3], r0, r1
	sub	r1, r3, r2
	add	@CPI23_1[0], r0, r1
	uma.heap_write	r1, r3, r0
	add	@CPI23_0[0], r0, r1
	uma.heap_write	r1, r2, r0
	add	1, r0, r2
	add	@CPI23_2[0], r0, r1
	uma.heap_write	r1, r2, r0
	jump	@.BB23_1
.func_end23:

revert_error_42b3090547df1d2001c96683413b8cf91c1b902ef5e3cb8d9f6f304cf7446f74:
.func_begin24:
	add	0, r0, r2
	add	@CPI24_0[0], r0, r1
	uma.heap_write	r1, r2, r0
	add	@CPI24_1[0], r0, r1
	uma.heap_write	r1, r2, r0
	jump	@.BB24_1
.BB24_1:
	add	0, r0, r3
	add	r3, r0, r1
	add	r3, r0, r2
	near_call	r0, @__cxa_throw, @DEFAULT_UNWIND
.func_end24:

zero_value_for_split_t_uint256:
.func_begin25:
	nop	stack+=[1]
	add	0, r0, stack-[1]
	jump	@.BB25_3
.BB25_3:
	add	stack-[1], r0, r1
	nop	stack-=[1]
	ret
.func_end25:

shift_right_0_unsigned:
.func_begin26:
	nop	stack+=[4]
	add	0, r0, stack-[4]
	add	r1, r0, stack-[3]
	add	stack-[3], r0, r1
	add	r1, r0, stack-[0]
	add	0, r0, r1
	sub!	r1, r1, r1
	jump.ne	@.BB26_4
	jump	@.BB26_5
.BB26_3:
	add	stack-[4], r0, r1
	nop	stack-=[4]
	ret
.BB26_4:
	add	0, r0, stack-[2]
	jump	@.BB26_6
.BB26_5:
	add	stack-[0], 0, r1
	add	r1, r0, stack-[2]
	jump	@.BB26_6
.BB26_6:
	add	stack-[2], r0, r1
	add	r1, r0, stack-[4]
	jump	@.BB26_3
.func_end26:

extract_from_storage_value_offset_0t_uint256:
.func_begin27:
	nop	stack+=[4]
	add	0, r0, stack-[4]
	add	r1, r0, stack-[3]
	add	stack-[3], r0, r1
.tmp91:
	near_call	r0, @shift_right_0_unsigned, @.BB27_2
.tmp92:
	add	r1, r0, stack-[1]
	jump	@.BB27_4
.BB27_2:
.tmp95:
	add	0, r0, r3
	add	r3, r0, r1
	add	r3, r0, r2
	near_call	r0, @__cxa_throw, @DEFAULT_UNWIND
.BB27_3:
	add	stack-[4], r0, r1
	nop	stack-=[4]
	ret
.BB27_4:
.tmp93:
	add	stack-[1], 0, r1
	near_call	r0, @cleanup_from_storage_t_uint256, @.BB27_2
.tmp94:
	add	r1, r0, stack-[0]
	jump	@.BB27_5
.BB27_5:
	add	stack-[0], 0, r1
	add	r1, r0, stack-[4]
	jump	@.BB27_3
.func_end27:

read_from_storage_split_offset_0_t_uint256:
.func_begin28:
	nop	stack+=[4]
	add	0, r0, stack-[4]
	add	r1, r0, stack-[3]
	add	stack-[3], r0, r1
.tmp96:
	near_call	r0, @__sload, @.BB28_2
.tmp97:
	add	r1, r0, stack-[1]
	jump	@.BB28_4
.BB28_2:
.tmp100:
	add	0, r0, r3
	add	r3, r0, r1
	add	r3, r0, r2
	near_call	r0, @__cxa_throw, @DEFAULT_UNWIND
.BB28_3:
	add	stack-[4], r0, r1
	nop	stack-=[4]
	ret
.BB28_4:
.tmp98:
	add	stack-[1], 0, r1
	near_call	r0, @extract_from_storage_value_offset_0t_uint256, @.BB28_2
.tmp99:
	add	r1, r0, stack-[0]
	jump	@.BB28_5
.BB28_5:
	add	stack-[0], 0, r1
	add	r1, r0, stack-[4]
	jump	@.BB28_3
.func_end28:

fun_get_11:
.func_begin29:
	nop	stack+=[6]
	add	0, r0, stack-[6]
.tmp101:
	near_call	r0, @zero_value_for_split_t_uint256, @.BB29_2
.tmp102:
	add	r1, r0, stack-[1]
	jump	@.BB29_4
.BB29_2:
.tmp105:
	add	0, r0, r3
	add	r3, r0, r1
	add	r3, r0, r2
	near_call	r0, @__cxa_throw, @DEFAULT_UNWIND
.BB29_3:
	add	stack-[6], r0, r1
	nop	stack-=[6]
	ret
.BB29_4:
	add	stack-[1], 0, r1
	add	r1, r0, stack-[5]
	add	stack-[5], r0, r1
	add	r1, r0, stack-[6]
.tmp103:
	add	0, r0, r1
	near_call	r0, @read_from_storage_split_offset_0_t_uint256, @.BB29_2
.tmp104:
	add	r1, r0, stack-[0]
	jump	@.BB29_5
.BB29_5:
	add	stack-[0], 0, r1
	add	r1, r0, stack-[4]
	add	stack-[4], r0, r1
	add	r1, r0, stack-[3]
	add	stack-[3], r0, r1
	add	r1, r0, stack-[6]
	jump	@.BB29_3
.func_end29:

cleanup_t_rational_1_by_1:
.func_begin30:
	nop	stack+=[2]
	add	0, r0, stack-[2]
	add	r1, r0, stack-[1]
	add	stack-[1], r0, r1
	add	r1, r0, stack-[2]
	jump	@.BB30_3
.BB30_3:
	add	stack-[2], r0, r1
	nop	stack-=[2]
	ret
.func_end30:

identity:
.func_begin31:
	nop	stack+=[2]
	add	0, r0, stack-[2]
	add	r1, r0, stack-[1]
	add	stack-[1], r0, r1
	add	r1, r0, stack-[2]
	jump	@.BB31_3
.BB31_3:
	add	stack-[2], r0, r1
	nop	stack-=[2]
	ret
.func_end31:

convert_t_rational_1_by_1_to_t_uint256:
.func_begin32:
	nop	stack+=[5]
	add	0, r0, stack-[5]
	add	r1, r0, stack-[4]
	add	stack-[4], r0, r1
.tmp106:
	near_call	r0, @cleanup_t_rational_1_by_1, @.BB32_2
.tmp107:
	add	r1, r0, stack-[2]
	jump	@.BB32_4
.BB32_2:
.tmp112:
	add	0, r0, r3
	add	r3, r0, r1
	add	r3, r0, r2
	near_call	r0, @__cxa_throw, @DEFAULT_UNWIND
.BB32_3:
	add	stack-[5], r0, r1
	nop	stack-=[5]
	ret
.BB32_4:
.tmp108:
	add	stack-[2], 0, r1
	near_call	r0, @identity, @.BB32_2
.tmp109:
	add	r1, r0, stack-[1]
	jump	@.BB32_5
.BB32_5:
.tmp110:
	add	stack-[1], 0, r1
	near_call	r0, @cleanup_t_uint256, @.BB32_2
.tmp111:
	add	r1, r0, stack-[0]
	jump	@.BB32_6
.BB32_6:
	add	stack-[0], 0, r1
	add	r1, r0, stack-[5]
	jump	@.BB32_3
.func_end32:

panic_error_0x11:
.func_begin33:
	add	@CPI33_0[0], r0, r1
	add	0, r0, r2
	uma.heap_write	r2, r1, r0
	add	17, r0, r3
	add	4, r0, r1
	uma.heap_write	r1, r3, r0
	add	@CPI33_1[0], r0, r1
	uma.heap_write	r1, r2, r0
	add	36, r0, r2
	add	@CPI33_2[0], r0, r1
	uma.heap_write	r1, r2, r0
	jump	@.BB33_1
.BB33_1:
	add	0, r0, r3
	add	r3, r0, r1
	add	r3, r0, r2
	near_call	r0, @__cxa_throw, @DEFAULT_UNWIND
.func_end33:

checked_add_t_uint256:
.func_begin34:
	nop	stack+=[5]
	add	0, r0, stack-[5]
	add	r1, r0, stack-[4]
	add	r2, r0, stack-[3]
	add	stack-[4], r0, r1
.tmp113:
	near_call	r0, @cleanup_t_uint256, @.BB34_2
.tmp114:
	add	r1, r0, stack-[1]
	jump	@.BB34_4
.BB34_2:
.tmp119:
	add	0, r0, r3
	add	r3, r0, r1
	add	r3, r0, r2
	near_call	r0, @__cxa_throw, @DEFAULT_UNWIND
.BB34_3:
	add	stack-[5], r0, r1
	nop	stack-=[5]
	ret
.BB34_4:
	add	stack-[1], 0, r1
	add	r1, r0, stack-[4]
	add	stack-[3], r0, r1
.tmp115:
	near_call	r0, @cleanup_t_uint256, @.BB34_2
.tmp116:
	add	r1, r0, stack-[0]
	jump	@.BB34_5
.BB34_5:
	add	stack-[0], 0, r1
	add	r1, r0, stack-[3]
	add	stack-[4], r0, r1
	add	stack-[3], r0, r2
	xor	@CPI34_0[0], r2, r2
	sub!	r1, r2, r1
	add	0, r0, r1
	add.gt	1, r0, r1
	and	1, r1, r1
	add	0, r0, r2
	sub!	r1, r2, r1
	jump.eq	@.BB34_7
	jump	@.BB34_6
.BB34_6:
.tmp117:
	near_call	r0, @panic_error_0x11, @.BB34_2
.tmp118:
	jump	@.BB34_8
.BB34_7:
	add	stack-[4], r0, r1
	add	stack-[3], r0, r2
	add	r1, r2, stack-[5]
	jump	@.BB34_3
.BB34_8:
	jump	@.BB34_7
.func_end34:

shift_left_0:
.func_begin35:
	nop	stack+=[4]
	add	0, r0, stack-[4]
	add	r1, r0, stack-[3]
	add	stack-[3], r0, r1
	add	r1, r0, stack-[0]
	add	0, r0, r1
	sub!	r1, r1, r1
	jump.ne	@.BB35_4
	jump	@.BB35_5
.BB35_3:
	add	stack-[4], r0, r1
	nop	stack-=[4]
	ret
.BB35_4:
	add	0, r0, stack-[2]
	jump	@.BB35_6
.BB35_5:
	add	stack-[0], 0, r1
	add	r1, r0, stack-[2]
	jump	@.BB35_6
.BB35_6:
	add	stack-[2], r0, r1
	add	r1, r0, stack-[4]
	jump	@.BB35_3
.func_end35:

update_byte_slice_32_shift_0:
.func_begin36:
	nop	stack+=[5]
	add	0, r0, stack-[5]
	add	r1, r0, stack-[4]
	add	r2, r0, stack-[3]
	add	@CPI36_0[0], r0, r1
	add	r1, r0, stack-[2]
	add	stack-[3], r0, r1
.tmp120:
	near_call	r0, @shift_left_0, @.BB36_2
.tmp121:
	add	r1, r0, stack-[0]
	jump	@.BB36_4
.BB36_2:
.tmp122:
	add	0, r0, r3
	add	r3, r0, r1
	add	r3, r0, r2
	near_call	r0, @__cxa_throw, @DEFAULT_UNWIND
.BB36_3:
	add	stack-[5], r0, r1
	nop	stack-=[5]
	ret
.BB36_4:
	add	stack-[0], 0, r1
	add	r1, r0, stack-[3]
	add	stack-[4], r0, r1
	add	stack-[2], r0, r2
	xor	@CPI36_0[0], r2, r2
	and	r1, r2, stack-[4]
	add	stack-[4], r0, r1
	add	stack-[3], r0, r2
	add	stack-[2], r0, r3
	and	r2, r3, r2
	or	r1, r2, stack-[5]
	jump	@.BB36_3
.func_end36:

convert_t_uint256_to_t_uint256:
.func_begin37:
	nop	stack+=[5]
	add	0, r0, stack-[5]
	add	r1, r0, stack-[4]
	add	stack-[4], r0, r1
.tmp123:
	near_call	r0, @cleanup_t_uint256, @.BB37_2
.tmp124:
	add	r1, r0, stack-[2]
	jump	@.BB37_4
.BB37_2:
.tmp129:
	add	0, r0, r3
	add	r3, r0, r1
	add	r3, r0, r2
	near_call	r0, @__cxa_throw, @DEFAULT_UNWIND
.BB37_3:
	add	stack-[5], r0, r1
	nop	stack-=[5]
	ret
.BB37_4:
.tmp125:
	add	stack-[2], 0, r1
	near_call	r0, @identity, @.BB37_2
.tmp126:
	add	r1, r0, stack-[1]
	jump	@.BB37_5
.BB37_5:
.tmp127:
	add	stack-[1], 0, r1
	near_call	r0, @cleanup_t_uint256, @.BB37_2
.tmp128:
	add	r1, r0, stack-[0]
	jump	@.BB37_6
.BB37_6:
	add	stack-[0], 0, r1
	add	r1, r0, stack-[5]
	jump	@.BB37_3
.func_end37:

prepare_store_t_uint256:
.func_begin38:
	nop	stack+=[2]
	add	0, r0, stack-[2]
	add	r1, r0, stack-[1]
	add	stack-[1], r0, r1
	add	r1, r0, stack-[2]
	jump	@.BB38_3
.BB38_3:
	add	stack-[2], r0, r1
	nop	stack-=[2]
	ret
.func_end38:

update_storage_value_offset_0t_uint256_to_t_uint256:
.func_begin39:
	nop	stack+=[8]
	add	r1, r0, stack-[8]
	add	r2, r0, stack-[7]
	add	stack-[7], r0, r1
.tmp130:
	near_call	r0, @convert_t_uint256_to_t_uint256, @.BB39_2
.tmp131:
	add	r1, r0, stack-[4]
	jump	@.BB39_4
.BB39_2:
.tmp140:
	add	0, r0, r3
	add	r3, r0, r1
	add	r3, r0, r2
	near_call	r0, @__cxa_throw, @DEFAULT_UNWIND
.BB39_3:
	nop	stack-=[8]
	ret
.BB39_4:
	add	stack-[4], 0, r1
	add	r1, r0, stack-[6]
	add	stack-[8], r0, r1
	add	r1, r0, stack-[2]
.tmp132:
	near_call	r0, @__sload, @.BB39_2
.tmp133:
	add	r1, r0, stack-[3]
	jump	@.BB39_5
.BB39_5:
	add	stack-[6], r0, r1
.tmp134:
	near_call	r0, @prepare_store_t_uint256, @.BB39_2
.tmp135:
	add	r1, r0, stack-[1]
	jump	@.BB39_6
.BB39_6:
.tmp136:
	add	stack-[1], 0, r2
	add	stack-[3], 0, r1
	near_call	r0, @update_byte_slice_32_shift_0, @.BB39_2
.tmp137:
	add	r1, r0, stack-[0]
	jump	@.BB39_7
.BB39_7:
.tmp138:
	add	stack-[2], 0, r2
	add	stack-[0], 0, r1
	near_call	r0, @__sstore, @.BB39_2
.tmp139:
	jump	@.BB39_8
.BB39_8:
	jump	@.BB39_3
.func_end39:

fun_inc_19:
.func_begin40:
	nop	stack+=[7]
	add	1, r0, stack-[7]
	add	stack-[7], r0, r1
.tmp141:
	near_call	r0, @convert_t_rational_1_by_1_to_t_uint256, @.BB40_2
.tmp142:
	add	r1, r0, stack-[2]
	jump	@.BB40_4
.BB40_2:
.tmp149:
	add	0, r0, r3
	add	r3, r0, r1
	add	r3, r0, r2
	near_call	r0, @__cxa_throw, @DEFAULT_UNWIND
.BB40_3:
	nop	stack-=[7]
	ret
.BB40_4:
	add	stack-[2], 0, r1
	add	r1, r0, stack-[6]
.tmp143:
	add	0, r0, r1
	near_call	r0, @read_from_storage_split_offset_0_t_uint256, @.BB40_2
.tmp144:
	add	r1, r0, stack-[1]
	jump	@.BB40_5
.BB40_5:
	add	stack-[1], 0, r1
	add	r1, r0, stack-[5]
	add	stack-[5], r0, r1
	add	stack-[6], r0, r2
.tmp145:
	near_call	r0, @checked_add_t_uint256, @.BB40_2
.tmp146:
	add	r1, r0, stack-[0]
	jump	@.BB40_6
.BB40_6:
	add	stack-[0], 0, r1
	add	r1, r0, stack-[4]
	add	stack-[4], r0, r2
.tmp147:
	add	0, r0, r1
	near_call	r0, @update_storage_value_offset_0t_uint256_to_t_uint256, @.BB40_2
.tmp148:
	jump	@.BB40_7
.BB40_7:
	jump	@.BB40_3
.func_end40:

checked_sub_t_uint256:
.func_begin41:
	nop	stack+=[5]
	add	0, r0, stack-[5]
	add	r1, r0, stack-[4]
	add	r2, r0, stack-[3]
	add	stack-[4], r0, r1
.tmp150:
	near_call	r0, @cleanup_t_uint256, @.BB41_2
.tmp151:
	add	r1, r0, stack-[1]
	jump	@.BB41_4
.BB41_2:
.tmp156:
	add	0, r0, r3
	add	r3, r0, r1
	add	r3, r0, r2
	near_call	r0, @__cxa_throw, @DEFAULT_UNWIND
.BB41_3:
	add	stack-[5], r0, r1
	nop	stack-=[5]
	ret
.BB41_4:
	add	stack-[1], 0, r1
	add	r1, r0, stack-[4]
	add	stack-[3], r0, r1
.tmp152:
	near_call	r0, @cleanup_t_uint256, @.BB41_2
.tmp153:
	add	r1, r0, stack-[0]
	jump	@.BB41_5
.BB41_5:
	add	stack-[0], 0, r1
	add	r1, r0, stack-[3]
	add	stack-[4], r0, r1
	add	stack-[3], r0, r2
	sub!	r1, r2, r1
	add	0, r0, r1
	add.lt	1, r0, r1
	and	1, r1, r1
	add	0, r0, r2
	sub!	r1, r2, r1
	jump.eq	@.BB41_7
	jump	@.BB41_6
.BB41_6:
.tmp154:
	near_call	r0, @panic_error_0x11, @.BB41_2
.tmp155:
	jump	@.BB41_8
.BB41_7:
	add	stack-[4], r0, r1
	add	stack-[3], r0, r2
	sub	r1, r2, stack-[5]
	jump	@.BB41_3
.BB41_8:
	jump	@.BB41_7
.func_end41:

fun_dec_27:
.func_begin42:
	nop	stack+=[7]
	add	1, r0, stack-[7]
	add	stack-[7], r0, r1
.tmp157:
	near_call	r0, @convert_t_rational_1_by_1_to_t_uint256, @.BB42_2
.tmp158:
	add	r1, r0, stack-[2]
	jump	@.BB42_4
.BB42_2:
.tmp165:
	add	0, r0, r3
	add	r3, r0, r1
	add	r3, r0, r2
	near_call	r0, @__cxa_throw, @DEFAULT_UNWIND
.BB42_3:
	nop	stack-=[7]
	ret
.BB42_4:
	add	stack-[2], 0, r1
	add	r1, r0, stack-[6]
.tmp159:
	add	0, r0, r1
	near_call	r0, @read_from_storage_split_offset_0_t_uint256, @.BB42_2
.tmp160:
	add	r1, r0, stack-[1]
	jump	@.BB42_5
.BB42_5:
	add	stack-[1], 0, r1
	add	r1, r0, stack-[5]
	add	stack-[5], r0, r1
	add	stack-[6], r0, r2
.tmp161:
	near_call	r0, @checked_sub_t_uint256, @.BB42_2
.tmp162:
	add	r1, r0, stack-[0]
	jump	@.BB42_6
.BB42_6:
	add	stack-[0], 0, r1
	add	r1, r0, stack-[4]
	add	stack-[4], r0, r2
.tmp163:
	add	0, r0, r1
	near_call	r0, @update_storage_value_offset_0t_uint256_to_t_uint256, @.BB42_2
.tmp164:
	jump	@.BB42_7
.BB42_7:
	jump	@.BB42_3
.func_end42:

__cxa_throw:
	revert

__sstore:
	sstore	r2, r1
	ret

__sload:
	sload	r1, r1
	ret

	.note.GNU-stack
	.rodata
CPI0_0:
	.cell 16777184
CPI0_1:
	.cell 16777152
CPI0_2:
	.cell 16777120
CPI1_0:
	.cell 16777184
CPI1_1:
	.cell 16777152
CPI2_0:
	.cell 16777152
CPI2_1:
	.cell 16777184
CPI2_2:
	.cell 107354813
CPI2_3:
	.cell 923993024
CPI2_4:
	.cell 1833756220
CPI2_5:
	.cell 3015506562
CPI4_0:
	.cell 16777184
CPI4_1:
	.cell 16777152
CPI8_0:
	.cell 16777184
CPI8_1:
	.cell 16777152
CPI9_0:
	.cell 16777184
CPI9_1:
	.cell 16777152
CPI10_0:
	.cell 57896044618658097711785492504343953926634992332820282019728792003956564819967
CPI19_0:
	.cell 16777152
CPI19_1:
	.cell 16777184
CPI19_2:
	.cell 16777120
CPI21_0:
	.cell 16777152
CPI21_1:
	.cell 16777184
CPI21_2:
	.cell 16777120
CPI22_0:
	.cell 16777152
CPI22_1:
	.cell 16777184
CPI22_2:
	.cell 16777120
CPI23_0:
	.cell 16777152
CPI23_1:
	.cell 16777184
CPI23_2:
	.cell 16777120
CPI24_0:
	.cell 16777184
CPI24_1:
	.cell 16777152
CPI33_0:
	.cell 35408467139433450592217433187231851964531694900788300625387963629091585785856
CPI33_1:
	.cell 16777184
CPI33_2:
	.cell 16777152
CPI34_0:
	.cell -1
CPI36_0:
	.cell -1