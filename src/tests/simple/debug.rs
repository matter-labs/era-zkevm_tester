use super::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manual() {
        let assembly: &'static str = r#"
        .text
        .file    "Test_16"
        .globl    __entry
    __entry:
    .func_begin0:
        nop    stack+=[1]
        add    @CPI0_0[0], r0, r4
        uma.heap_write    r4, r1, r0
        add    @CPI0_1[0], r0, r1
        uma.heap_write    r1, r2, r0
        and    1, r3, r2
        add    0, r0, r1
        sub!    r2, r1, r2
        jump.ne    @.BB0_3
        jump    @.BB0_4
    .BB0_3:
        add    128, r0, r2
        add    64, r0, r3
        uma.heap_write    r3, r2, r0
        add    @CPI0_0[0], r0, r3
        uma.heap_write    r3, r2, r0
        add    @CPI0_1[0], r0, r2
        uma.heap_write    r2, r1, r0
        jump    @.BB0_2
    .BB0_4:
    .tmp0:
        near_call    r0, @__selector, @.BB0_1
    .tmp1:
    .BB0_2:
        add    @CPI0_0[0], r0, r1
        uma.heap_read    r1, r0, r1
        add    @CPI0_1[0], r0, r2
        uma.heap_read    r2, r0, r2
        shl.s    32, r2, r2
        add    r2, r1, r1
        nop    stack-=[1]
        ret
    .BB0_1:
    .tmp2:
        add    96, r0, r1
        uma.heap_read    r1, r0, r1
        add    1, r0, r2
        sub!    r1, r2, r1
        jump.eq    @.BB0_2
        add    @CPI0_0[0], r0, r1
        uma.heap_read    r1, r0, r1
        add    @CPI0_1[0], r0, r2
        uma.heap_read    r2, r0, r2
        shl.s    32, r2, r2
        add    r2, r1, stack-[1]
        context.sp    r1
        add    0, r1, r1
        revert
    .func_end0:
    
    __selector:
    .func_begin1:
        add    128, r0, r2
        add    64, r0, r3
        uma.heap_write    r3, r2, r0
        add    0, r0, r1
        add    @CPI1_0[0], r0, r4
        uma.heap_read    r4, r0, r5
        add    3, r0, r4
        sub!    r5, r4, r4
        add    r1, r0, r6
        jump.le    @.BB1_1
        add    @CPI1_1[0], r0, r7
        add    @CPI1_2[0], r0, r4
        uma.heap_read    r4, r0, r4
        uma.calldata_read    r4, r0, r6
        and    @CPI1_3[0], r6, r6
        add    @CPI1_4[0], r0, r8
        sub!    r6, r8, r6
        add    r1, r0, r6
        jump.ne    @.BB1_1
        add    r5, r7, r5
        add    @CPI1_5[0], r0, r6
        sub!    r5, r3, r3
        add    0, r0, r7
        add.lt    r6, r0, r7
        and    r5, r6, r5
        add    0, r0, r3
        sub!    r5, r3, r8
        add    0, r0, r8
        add.gt    r6, r0, r8
        sub!    r5, r6, r5
        add    r7, r0, r5
        add.eq    r8, r0, r5
        sub!    r5, r3, r5
        add    r1, r0, r6
        jump.ne    @.BB1_1
        add    4, r4, r5
        uma.calldata_read    r5, r0, r5
        add    255, r0, r6
        sub!    r5, r6, r7
        jump.gt    @.BB1_5
        add    36, r4, r4
        uma.calldata_read    r4, r0, r4
        sub!    r4, r6, r6
        jump.gt    @.BB1_7
        xor    255, r4, r6
        sub!    r5, r6, r6
        jump.gt    @.BB1_9
        add    r4, r5, r1
        and    255, r1, r1
        uma.heap_write    r2, r1, r0
        add    @CPI1_2[0], r0, r1
        uma.heap_write    r1, r2, r0
        add    32, r0, r1
        add    @CPI1_0[0], r0, r2
        uma.heap_write    r2, r1, r0
        ret
    .BB1_9:
        add    @CPI1_6[0], r0, r2
        uma.heap_write    r3, r2, r0
        add    17, r0, r2
        add    4, r0, r3
        uma.heap_write    r3, r2, r0
        add    36, r0, r6
    .BB1_1:
        add    @CPI1_2[0], r0, r2
        uma.heap_write    r2, r1, r0
        add    @CPI1_0[0], r0, r2
        uma.heap_write    r2, r6, r0
        revert
    .BB1_5:
        add    @CPI1_2[0], r0, r1
        uma.heap_write    r1, r3, r0
        add    @CPI1_0[0], r0, r1
        uma.heap_write    r1, r3, r0
    .BB1_7:
        add    @CPI1_2[0], r0, r1
        uma.heap_write    r1, r3, r0
        add    @CPI1_0[0], r0, r1
        uma.heap_write    r1, r3, r0
    .func_end1:
    
        .note.GNU-stack
        .rodata
    CPI0_0:
        .cell 16777184
    CPI0_1:
        .cell 16777152
    CPI1_0:
        .cell 16777152
    CPI1_1:
        .cell -4
    CPI1_2:
        .cell 16777184
    CPI1_3:
        .cell -26959946667150639794667015087019630673637144422540572481103610249216
    CPI1_4:
        .cell -36850303905235440479582801831662192552660019583232577374996976515539611418624
    CPI1_5:
        .cell -57896044618658097711785492504343953926634992332820282019728792003956564819968
    CPI1_6:
        .cell 35408467139433450592217433187231851964531694900788300625387963629091585785856
                "#;

        run_for_result_only(assembly);
    }
}