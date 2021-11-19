use super::*;
use super::default_environment::*;
use zk_evm::testing::create_default_testing_tools;
use zk_evm::testing::debug_tracer::{NoopTracer, ClosureBasedTracer};
use zk_evm::testing::get_final_net_states;
use super::PartialVmState;
use zk_evm::testing::event_sink::EventMessage;
use std::collections::HashMap;

pub fn run_compiled_assembly(assembly: Vec<[u8; 32]>, calldata: Vec<[u8; 32]>, num_cycles: usize)
-> (
    Vec<LogQuery>,
    [HashMap<Address, HashMap<U256, U256>>; zk_evm::testing::NUM_SHARDS],
    Vec<LogQuery>,
    Vec<EventMessage>,
    Vec<EventMessage>,
    SimpleMemory,
){
    let mut tools = create_default_testing_tools();
    let block_properties = create_default_block_properties();
    let mut vm = create_vm_with_default_settings(&mut tools, &block_properties);

    // manually encode LE
    let mut opcodes = vec![];
    for el in assembly.into_iter() {
        let word = U256::from_little_endian(&el);
        opcodes.push(word);
    }
    let mut calldata_words = vec![];
    for el in calldata.into_iter() {
        let word = U256::from_big_endian(&el);
        calldata_words.push(word);
    }

    vm.callstack.get_current_stack_mut().calldata_len = MemoryOffset(calldata_words.len() as u16);

    vm.memory.populate(vec![
        (ENTRY_POINT_PAGE, opcodes),
        (CALLDATA_PAGE, calldata_words),
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
    let (full_storage_access_history, storage_per_shard, events_log_history, events, l1_messages, memory) = get_final_net_states(tools);
    
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

    (full_storage_access_history, storage_per_shard, events_log_history, events, l1_messages, memory)
}

pub fn run_text_assembly(assembly: String, calldata: Vec<[u8; 32]>, num_cycles: usize) 
-> (
    Vec<LogQuery>,
    [HashMap<Address, HashMap<U256, U256>>; zk_evm::testing::NUM_SHARDS],
    Vec<LogQuery>,
    Vec<EventMessage>,
    Vec<EventMessage>,
    SimpleMemory,
){
    let assembly = Assembly::try_from(assembly).expect("must get a valid assembly as the input");
    dbg!(&assembly.instructions);
    let compiled = assembly.compile_to_bytecode(); 
    for el in compiled.iter().take(16) {
        println!("{}", hex::encode(el));
    }
    run_compiled_assembly(compiled, calldata, num_cycles)
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

   run_text_assembly(asm_text.to_owned(), vec![], 2);
}

pub(crate) fn pretty_print_memory_dump(content: &Vec<[u8; 32]>, range: std::ops::Range<u16>) {
    println!("Memory dump:");
    println!("-----------------------------------------");
    for (cont, index) in content.into_iter().zip(range.into_iter()) {
        println!("{:04x}: 0x{}", index, hex::encode(cont));
    }
    println!("-----------------------------------------");
}