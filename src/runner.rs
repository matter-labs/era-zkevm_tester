use super::*;
use super::default_environment::*;
use zk_evm::testing::create_default_testing_tools;
use zk_evm::testing::debug_tracer::{NoopTracer, ClosureBasedTracer};
use zk_evm::testing::get_final_net_states;
use super::PartialVmState;

pub fn run_compiled_assembly(assembly: Vec<[u8; 32]>, num_cycles: usize) {
    let mut tools = create_default_testing_tools();
    let block_properties = create_default_block_properties();
    let mut vm = create_vm_with_default_settings(&mut tools, &block_properties);

    // manually encode LE
    let mut opcodes = vec![];
    for el in assembly.into_iter() {
        let word = U256::from_little_endian(&el);
        opcodes.push(word);
    }
    vm.memory.populate(vec![
        (ENTRY_POINT_PAGE, opcodes)
    ]);

    let mut noop_tracer = NoopTracer::new();

    for _ in 0..(num_cycles-1) {
        vm.cycle(&mut noop_tracer);
    }

    let mut final_state = None;
    let handle = &mut final_state;
    let mut debug_tracer = ClosureBasedTracer::new( move |a, b, _c| {
        let f = PartialVmState {
            skip_cycle: b.skip_cycle,
            error_flags_collection: b.error_flags_collection,
            final_masked_opcode: b.final_masked_opcode,
            resolved_jump_condition: b.resolved_jump_condition,
            registers: a.registers.clone(),
            flags: a.flags.clone(),
            timestamp: a.timestamp,    
            memory_page_counter: a.memory_page_counter,
            tx_number_in_block: a.tx_number_in_block,
            pending_port: a.pending_port.clone(),
            pending_cycles_left: a.pending_cycles_left,
            tx_origin: a.tx_origin.clone(), // large one
            callstack: a.callstack.clone(),
        };
        *handle = Some(f);
    }); 

    vm.cycle(&mut debug_tracer);
    drop(debug_tracer);
    drop(vm);

    let final_state = final_state.unwrap();

    println!("Final summary: \n{:?}", final_state);
    let (full_storage_access_history, storage_pre_shard, events_log_history, events, l1_messages) = get_final_net_states(tools);
    
    println!("------------------------------------------------------");
    println!("Storage log access history:");
    println!("{:?}", full_storage_access_history);
    println!("Event log access history:");
    println!("{:?}", events_log_history);

    println!("------------------------------------------------------");
    println!("Net events:");
    println!("{:?}", events);
    println!("Net L1 messages:");
    println!("{:?}", l1_messages);
}

pub fn run_text_assembly(assembly: String, num_cycles: usize) {
    let assembly = Assembly::try_from(assembly).expect("must get a valid assembly as the input");
    run_compiled_assembly(assembly.compile_to_bytecode(), num_cycles);
}

#[test]
fn test_trivial() {
    let asm_text = r#"
    .text
       add .const_1, r2, r3
       ret r0, r0
    .data
    const_1:
       .cell 777 ; 2 bytes
   "#;

   run_text_assembly(asm_text.to_owned(), 2);
}