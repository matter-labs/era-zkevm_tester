use zk_evm::vm_state::VmState;

use super::*;

pub(crate) const TIC_TAC_TOE_ASM: &'static str = r#"
    .text
    main:
        mov calldata[0], r1
        ret r0, r0
    ret_with_error:
        revert r0, r0
"#;

#[test]
fn run_tic_tac_toe() {
    let mut input = [0u8; 32];
    input[1] = 1;
    let (full_storage_access_history, storage_per_shard, events_log_history, events, l1_messages, memory) = 
        run_text_assembly(TIC_TAC_TOE_ASM.to_owned(), vec![input], 3);
    let range = 0u16..16u16;
    let heap_page = CallStackEntry::heap_page_from_base(MemoryPage(INITIAL_BASE_PAGE));
    println!("Heap dump");
    let heap_content = memory.dump_page_content(heap_page.0, range.clone());
    pretty_print_memory_dump(&heap_content, range.clone());

    println!("Stack dump");
    let stack_page = CallStackEntry::heap_page_from_base(MemoryPage(INITIAL_BASE_PAGE));
    let stack_content = memory.dump_page_content(stack_page.0, range.clone());
    pretty_print_memory_dump(&stack_content, range.clone());

    println!("Calldata dump");
    let calldata_content = memory.dump_page_content(CALLDATA_PAGE, range.clone());
    pretty_print_memory_dump(&calldata_content, range.clone());
}