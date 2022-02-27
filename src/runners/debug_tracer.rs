use zk_evm::{abstractions::*, testing::memory::SimpleMemory};
use zkevm_assembly::Assembly;

use crate::runners::compiler_tests::get_debug;

#[derive(Debug)]
pub struct DebugTracer;

impl Tracer for DebugTracer {
    const CALL_BEFORE_DECODING: bool = true;
    const CALL_AFTER_DECODING: bool = true;
    const CALL_BEFORE_EXECUTION: bool = true;
    const CALL_AFTER_EXECUTION: bool = true;

    type SupportedMemory = SimpleMemory;

    fn before_decoding(&mut self, state: VmLocalStateData<'_>, memory: &Self::SupportedMemory) {
        if !get_debug() {
            return;
        }
        dbg!(state);
    }
    fn after_decoding(
        &mut self,
        state: VmLocalStateData<'_>,
        data: AfterDecodingData,
        memory: &Self::SupportedMemory,
    ) {
        if !get_debug() {
            return;
        }
        dbg!(data);
    }
    fn before_execution(
        &mut self,
        state: VmLocalStateData<'_>,
        data: BeforeExecutionData,
        memory: &Self::SupportedMemory,
    ) {
        if !get_debug() {
            return;
        }
        dbg!(data);
    }
    fn after_execution(
        &mut self,
        state: VmLocalStateData<'_>,
        data: AfterExecutionData,
        memory: &Self::SupportedMemory,
    ) {
        if !get_debug() {
            return;
        }
        dbg!(data);
    }
}

#[derive(Debug)]
pub struct DebugTracerWithAssembly<'a> {
    pub assembly: &'a Assembly,
}

impl<'a> Tracer for DebugTracerWithAssembly<'a> {
    const CALL_BEFORE_DECODING: bool = true;
    const CALL_AFTER_DECODING: bool = true;
    const CALL_BEFORE_EXECUTION: bool = true;
    const CALL_AFTER_EXECUTION: bool = true;

    type SupportedMemory = SimpleMemory;

    fn before_decoding(&mut self, state: VmLocalStateData<'_>, memory: &Self::SupportedMemory) {
        if !get_debug() {
            return;
        }
        println!("New cycle -------------------------");
        // dbg!(state);
        let pc = state.vm_local_state.callstack.get_current_stack().pc;
        if let Some(line) = self.assembly.pc_line_mapping.get(&(pc as usize)).copied() {
            let l = if line == 0 {
                self.assembly.assembly_code.lines().next().unwrap()
            } else {
                self.assembly
                    .assembly_code
                    .lines()
                    .skip(line)
                    .next()
                    .unwrap()
            };

            println!("Executing {}", l.trim());
        }
    }
    fn after_decoding(
        &mut self,
        state: VmLocalStateData<'_>,
        data: AfterDecodingData,
        memory: &Self::SupportedMemory,
    ) {
        if !get_debug() {
            return;
        }
        // println!(
        //     "Raw opcode LE: 0x{}",
        //     hex::encode(&data.raw_opcode_unmasked.to_le_bytes())
        // );
        // dbg!(data);
    }
    fn before_execution(
        &mut self,
        state: VmLocalStateData<'_>,
        data: BeforeExecutionData,
        memory: &Self::SupportedMemory,
    ) {
        if !get_debug() {
            return;
        }
        // dbg!(data);
    }
    fn after_execution(
        &mut self,
        state: VmLocalStateData<'_>,
        data: AfterExecutionData,
        memory: &Self::SupportedMemory,
    ) {
        if !get_debug() {
            return;
        }
        // dbg!(data);
        println!("Registers: {:?}", state.vm_local_state.registers.iter().map(|el| format!("{:x}", el)).collect::<Vec<_>>());
    }
}
