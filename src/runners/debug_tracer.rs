use zk_evm::{abstractions::*, testing::memory::SimpleMemory, vm_state::CallStackEntry, u256_to_address_unchecked};
use zkevm_assembly::Assembly;

use crate::runners::compiler_tests::{get_tracing_mode, VmTracingOptions};

use super::hashmap_based_memory::SimpleHashmapMemory;

#[derive(Debug)]
pub struct DummyVmTracer;

impl Tracer for DummyVmTracer {
    const CALL_BEFORE_DECODING: bool = true;
    const CALL_AFTER_DECODING: bool = true;
    const CALL_BEFORE_EXECUTION: bool = true;
    const CALL_AFTER_EXECUTION: bool = true;

    type SupportedMemory = SimpleMemory;

    fn before_decoding(&mut self, state: VmLocalStateData<'_>, _memory: &Self::SupportedMemory) {
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

use crate::Address;

#[derive(Debug)]
pub struct DebugTracerWithAssembly {
    pub current_code_address: Address,
    pub code_address_to_assembly: std::collections::HashMap<Address, Assembly>,
}

impl Tracer for DebugTracerWithAssembly {
    const CALL_BEFORE_DECODING: bool = true;
    const CALL_AFTER_DECODING: bool = true;
    const CALL_BEFORE_EXECUTION: bool = true;
    const CALL_AFTER_EXECUTION: bool = true;

    type SupportedMemory = SimpleHashmapMemory;

    fn before_decoding(&mut self, state: VmLocalStateData<'_>, _memory: &Self::SupportedMemory) {
        if get_tracing_mode() != VmTracingOptions::ManualVerbose {
            return;
        }
        println!("New cycle -------------------------");
        let pc = state.vm_local_state.callstack.get_current_stack().pc;
        if let Some(assembly) = self.code_address_to_assembly.get(&self.current_code_address) {
            if let Some(line) = assembly.pc_line_mapping.get(&(pc as usize)).copied() {
                let l = if line == 0 {
                    assembly.assembly_code.lines().next().unwrap()
                } else {
                    assembly
                        .assembly_code
                        .lines()
                        .skip(line)
                        .next()
                        .unwrap()
                };
    
                println!("Executing {}", l.trim());
                // if l.trim().contains("far_call") {
                //     println!("Breakpoint");
                // }
            }
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
        state: VmLocalStateData<'_>,
        data: BeforeExecutionData,
        memory: &Self::SupportedMemory,
    ) {
        if get_tracing_mode() != VmTracingOptions::ManualVerbose {
            return;
        }

        use zk_evm::zkevm_opcode_defs::*;

        match data.opcode.variant.opcode {
            Opcode::Ret(inner_variant) => {
                if !state.vm_local_state.callstack.get_current_stack().is_local_frame {
                    // catch returndata
                    if inner_variant == RetOpcode::Ok || inner_variant == RetOpcode::Revert {
                        let src0 = data.src0_value;

                        let abi = RetABI::from_u256(src0);
                        let page = if abi.transit_page {
                            state.vm_local_state.callstack.returndata_page
                        } else {
                            CallStackEntry::heap_page_from_base(state.vm_local_state.callstack.get_current_stack().base_memory_page)
                        };

                        let returndata = crate::runners::compiler_tests::dump_memory_page_by_offset_and_length(
                            memory,
                            page.0,
                            abi.returndata_offset.into_raw() as usize,
                            abi.returndata_length.into_raw() as usize,
                        );

                        println!("Performed return/revert with {} bytes with 0x{}", returndata.len(), hex::encode(&returndata));
                    } else {
                        println!("Returned with PANIC");
                    }
                }
            },
            Opcode::FarCall(_) => {
                // catch calldata
                let src0 = data.src0_value;
                let dest = u256_to_address_unchecked(&src0);
                let src1 = data.src1_value;

                let abi = FarCallABI::from_u256(src1);
                let page = if abi.transit_page {
                    state.vm_local_state.callstack.get_current_stack().calldata_page
                } else {
                    CallStackEntry::heap_page_from_base(state.vm_local_state.callstack.get_current_stack().base_memory_page)
                };

                let calldata = crate::runners::compiler_tests::dump_memory_page_by_offset_and_length(
                    memory,
                    page.0,
                    abi.calldata_offset.into_raw() as usize,
                    abi.calldata_length.into_raw() as usize,
                );

                println!("Performed far_call to {:?} with {} bytes with 0x{}", dest, calldata.len(), hex::encode(&calldata));
            },
            _ => {}
        }
    }
    fn after_execution(
        &mut self,
        state: VmLocalStateData<'_>,
        _data: AfterExecutionData,
        _memory: &Self::SupportedMemory,
    ) {
        self.current_code_address = state.vm_local_state.callstack.get_current_stack().code_address;
        if get_tracing_mode() != VmTracingOptions::ManualVerbose {
            return;
        }
        println!(
            "Registers: {:?}",
            state
                .vm_local_state
                .registers
                .iter()
                .map(|el| format!("0x{:064x}", el))
                .collect::<Vec<_>>()
        );
    }
}
