use zk_evm::vm_state::VmState;

use super::*;

pub(crate) const TIC_TAC_TOE_ASM: &'static str = r#"
    .text
    main:
        mov calldata[0], r1; r1 is a transcript
        mov #0, r2; step counter
        mov #0, r3; player counter
        mov #0, r15; error marker
        jump .subcycle
    subcycle:
        sub.s #9, r2, r0; // check if it's the end
        jump.lt r0, .cycle; // continue or not
        jump .check_winning; // else - return
    cycle:
        div.s #9, r1, r1, r4; // get cell into r4
        jump .mark_cell
    cycle_finish:
        jz r15, .cycle_finish_ok;
        jump .ret_with_error;
    cycle_finish_ok:
        add #1, r2, r2; // increment step counter 
        jump .subcycle;
    mark_cell:
        mov heap[r4 + 4], r5; // load from static heap array
        jz r5, .mark_ok, .mark_err
    mark_ok:
        mov #4, r3; // by default player 2
        div.s #2, r2, r0, r6; // get step % 2
        mov #1, r7;
        cmov.z r6, r7, r3;
        mov r3, heap[r4+4]; // put current player marker
        jump .cycle_finish;
    mark_err:
        mov #1, r15;
        jump .cycle_finish;
    ret_with_error:
        revert r0, r0
    check_winning:
        mov #0, r2; step counter
        jump .winning_subcycle
    winning_subcycle:
        sub.s #8, r2, r0; // check if it's the end
        jump.lt r0, .winning_main; // continue or not
        jump .ret_ok; // else - return
    winning_main:
        mov .data[r2], r4; // idx
        mov .data[r2 + 1], r5; // stride
        mov heap[r4+4], r6; // count points
        add r4, r5, r4;
        add heap[r4+4], r6, r6;
        add r4, r5, r4;
        add heap[r4+4], r6, r6;
        mov #1, r8;
        mov #0, r7;
        sub #3, r6, r0;
        cmov.eq r0, r8, r7;
        jump.eq r0, .ret_ok;
        mov #2, r8;
        sub #12, r6, r0;
        cmov.eq r0, r8, r7;
        jump.eq r0, .ret_ok;
        jump .winning_subcycle
    ret_ok:
        mov r7, heap[0];
        ret r0, r0;
    .data
    offsets_and_strides:
        .uint256 0;
        .uint256 1;
        .uint256 3;
        .uint256 1;
        .uint256 6;
        .uint256 1;
        .uint256 0;
        .uint256 3;
        .uint256 1;
        .uint256 3;
        .uint256 2;
        .uint256 3;
        .uint256 0;
        .uint256 4;
        .uint256 2;
        .uint256 2;
"#;

use zk_evm::ethereum_types::U256;

#[test]
fn run_tic_tac_toe() {
    let mut input = [0u8; 32];
    let mut transcript = U256::zero();
    let nine = U256::from(9u64);
    transcript += U256::from(8u64);
    transcript *= nine;
    transcript += U256::from(6u64);
    transcript *= nine;
    transcript += U256::from(5u64);
    transcript *= nine;
    transcript += U256::from(2u64);
    transcript *= nine;
    transcript += U256::from(7u64);
    transcript *= nine;
    transcript += U256::from(3u64);
    transcript *= nine;
    transcript += U256::from(1u64);
    transcript *= nine;
    transcript += U256::from(0u64);
    transcript *= nine;
    transcript += U256::from(4u64);

    transcript.to_big_endian(&mut input);
    let (full_storage_access_history, storage_per_shard, events_log_history, events, l1_messages, memory) = 
        run_text_assembly(TIC_TAC_TOE_ASM.to_owned(), vec![input], 500);
    let range = 0u16..16u16;
    let heap_page = CallStackEntry::heap_page_from_base(MemoryPage(INITIAL_BASE_PAGE));
    println!("Heap dump");
    let heap_content = memory.dump_page_content(heap_page.0, range.clone());
    pretty_print_memory_dump(&heap_content, range.clone());

    println!("Stack dump");
    let stack_page = CallStackEntry::stack_page_from_base(MemoryPage(INITIAL_BASE_PAGE));
    let stack_content = memory.dump_page_content(stack_page.0, range.clone());
    pretty_print_memory_dump(&stack_content, range.clone());

    println!("Calldata dump");
    let calldata_content = memory.dump_page_content(CALLDATA_PAGE, range.clone());
    pretty_print_memory_dump(&calldata_content, range.clone());
}