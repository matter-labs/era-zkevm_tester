
use zk_evm::abstractions::*;
use zkevm_assembly::Assembly;

#[derive(Debug)]
pub struct DebugTracer;


impl Tracer for DebugTracer {
    const CALL_BEFORE_DECODING: bool = true;
    const CALL_AFTER_DECODING: bool = true;
    const CALL_BEFORE_EXECUTION: bool = true;
    const CALL_AFTER_EXECUTION: bool = true;

    fn before_decoding(&mut self, state: VmLocalStateData<'_>) {
        dbg!(state);
    }
    fn after_decoding(&mut self, state: VmLocalStateData<'_>, data: AfterDecodingData) {
        dbg!(data);
    }
    fn before_execution(&mut self, state: VmLocalStateData<'_>, data: BeforeExecutionData) {
        dbg!(data);
    }
    fn after_execution(&mut self, state: VmLocalStateData<'_>, data: AfterExecutionData) {
        dbg!(data);
    }
}

#[derive(Debug)]
pub struct DebugTracerWithAssembly<'a> {
    pub assembly: &'a Assembly
}


impl<'a> Tracer for DebugTracerWithAssembly<'a> {
    const CALL_BEFORE_DECODING: bool = true;
    const CALL_AFTER_DECODING: bool = true;
    const CALL_BEFORE_EXECUTION: bool = true;
    const CALL_AFTER_EXECUTION: bool = true;

    fn before_decoding(&mut self, state: VmLocalStateData<'_>) {
        println!("New cycle -------------------------");
        dbg!(state);
        let pc = state.vm_local_state.callstack.get_current_stack().pc;
        if let Some(line) = self.assembly.pc_line_mapping.get(&(pc as usize)).copied() {
            let l = if line == 0 {
                self.assembly.assembly_code.lines().next().unwrap()
            } else {
                self.assembly.assembly_code.lines().skip(line).next().unwrap()
            };

            println!("Executing {}", l.trim());
        } 
    }
    fn after_decoding(&mut self, state: VmLocalStateData<'_>, data: AfterDecodingData) {
        println!("Raw opcode LE: 0x{}", hex::encode(&data.raw_opcode_unmasked.to_le_bytes()));
        dbg!(data);
    }
    fn before_execution(&mut self, state: VmLocalStateData<'_>, data: BeforeExecutionData) {
        dbg!(data);
    }
    fn after_execution(&mut self, state: VmLocalStateData<'_>, data: AfterExecutionData) {
        dbg!(data);
    }
}