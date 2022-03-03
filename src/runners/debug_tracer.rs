use zk_evm::{abstractions::*, testing::memory::SimpleMemory};
use zkevm_assembly::Assembly;

use crate::runners::compiler_tests::{get_tracing_mode, VmTracingOptions};

#[derive(Debug)]
pub struct DummyVmTracer;

impl Tracer for DummyVmTracer {
    const CALL_BEFORE_DECODING: bool = true;
    const CALL_AFTER_DECODING: bool = true;
    const CALL_BEFORE_EXECUTION: bool = true;
    const CALL_AFTER_EXECUTION: bool = true;

    type SupportedMemory = SimpleMemory;

    fn before_decoding(
        &mut self,
        state: VmLocalStateData<'_>,
        _memory: &Self::SupportedMemory
    ) {
        if get_tracing_mode() != VmTracingOptions::ManualVerbose {
            return;
        }
        dbg!(state);
    }
    fn after_decoding(
        &mut self,
        _state: VmLocalStateData<'_>,
        data: AfterDecodingData,
        _memory: &Self::SupportedMemory,
    ) {
        if get_tracing_mode() != VmTracingOptions::ManualVerbose {
            return;
        }
        dbg!(data);
    }
    fn before_execution(
        &mut self,
        _state: VmLocalStateData<'_>,
        data: BeforeExecutionData,
        _memory: &Self::SupportedMemory,
    ) {
        if get_tracing_mode() != VmTracingOptions::ManualVerbose {
            return;
        }
        dbg!(data);
    }
    fn after_execution(
        &mut self,
        _state: VmLocalStateData<'_>,
        data: AfterExecutionData,
        _memory: &Self::SupportedMemory,
    ) {
        if get_tracing_mode() != VmTracingOptions::ManualVerbose {
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

    fn before_decoding(&mut self, state: VmLocalStateData<'_>, _memory: &Self::SupportedMemory) {
        if get_tracing_mode() != VmTracingOptions::ManualVerbose {
            return;
        }
        println!("New cycle -------------------------");
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
        _state: VmLocalStateData<'_>,
        _data: AfterDecodingData,
        _memory: &Self::SupportedMemory,
    ) {
        if get_tracing_mode() != VmTracingOptions::ManualVerbose {
            return;
        }
    }
    fn before_execution(
        &mut self,
        _state: VmLocalStateData<'_>,
        _data: BeforeExecutionData,
        _memory: &Self::SupportedMemory,
    ) {
        if get_tracing_mode() != VmTracingOptions::ManualVerbose {
            return;
        }
    }
    fn after_execution(
        &mut self,
        state: VmLocalStateData<'_>,
        _data: AfterExecutionData,
        _memory: &Self::SupportedMemory,
    ) {
        if get_tracing_mode() != VmTracingOptions::ManualVerbose {
            return;
        }
        println!(
            "Registers: {:?}",
            state
                .vm_local_state
                .registers
                .iter()
                .map(|el| format!("{:x}", el))
                .collect::<Vec<_>>()
        );
    }
}
