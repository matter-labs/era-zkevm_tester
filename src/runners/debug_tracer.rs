use zk_evm::{
    abstractions::*,
    reference_impls::memory::SimpleMemory,
    u256_to_address_unchecked,
    vm_state::CallStackEntry,
    zkevm_opcode_defs::decoding::{AllowedPcOrImm, EncodingModeProduction, VmEncodingMode},
};
use zkevm_assembly::Assembly;
use zk_evm::tracing::*;

use crate::runners::compiler_tests::{get_tracing_mode, VmTracingOptions};

use super::hashmap_based_memory::SimpleHashmapMemory;

#[derive(Debug)]
pub struct DummyVmTracer<const N: usize = 8, E: VmEncodingMode<N> = EncodingModeProduction> {
    _marker: std::marker::PhantomData<E>,
}

impl<const N: usize, E: VmEncodingMode<N>> DummyVmTracer<N, E> {
    pub fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

impl<const N: usize, E: VmEncodingMode<N>> Tracer<N, E> for DummyVmTracer<N, E> {
    const CALL_BEFORE_DECODING: bool = true;
    const CALL_AFTER_DECODING: bool = true;
    const CALL_BEFORE_EXECUTION: bool = true;
    const CALL_AFTER_EXECUTION: bool = true;

    type SupportedMemory = SimpleMemory;

    fn before_decoding(
        &mut self,
        state: VmLocalStateData<'_, N, E>,
        _memory: &Self::SupportedMemory,
    ) {
        if get_tracing_mode() != VmTracingOptions::ManualVerbose {
            return;
        }
        dbg!(state);
    }
    fn after_decoding(
        &mut self,
        _state: VmLocalStateData<'_, N, E>,
        data: AfterDecodingData<N, E>,
        _memory: &Self::SupportedMemory,
    ) {
        if get_tracing_mode() != VmTracingOptions::ManualVerbose {
            return;
        }
        dbg!(data);
    }
    fn before_execution(
        &mut self,
        _state: VmLocalStateData<'_, N, E>,
        data: BeforeExecutionData<N, E>,
        _memory: &Self::SupportedMemory,
    ) {
        if get_tracing_mode() != VmTracingOptions::ManualVerbose {
            return;
        }
        dbg!(data);
    }
    fn after_execution(
        &mut self,
        _state: VmLocalStateData<'_, N, E>,
        data: AfterExecutionData<N, E>,
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
pub struct DebugTracerWithAssembly<
    const N: usize = 8,
    E: VmEncodingMode<N> = EncodingModeProduction,
> {
    pub current_code_address: Address,
    pub code_address_to_assembly: std::collections::HashMap<Address, Assembly>,
    pub _marker: std::marker::PhantomData<E>,
}

impl<const N: usize, E: VmEncodingMode<N>> Tracer<N, E> for DebugTracerWithAssembly<N, E> {
    const CALL_BEFORE_DECODING: bool = true;
    const CALL_AFTER_DECODING: bool = true;
    const CALL_BEFORE_EXECUTION: bool = true;
    const CALL_AFTER_EXECUTION: bool = true;

    type SupportedMemory = SimpleHashmapMemory;

    fn before_decoding(
        &mut self,
        state: VmLocalStateData<'_, N, E>,
        _memory: &Self::SupportedMemory,
    ) {
        if get_tracing_mode() != VmTracingOptions::ManualVerbose {
            return;
        }
        println!("New cycle -------------------------");
        let pc = state.vm_local_state.callstack.get_current_stack().pc;
        if let Some(assembly) = self
            .code_address_to_assembly
            .get(&self.current_code_address)
        {
            if let Some(line) = assembly
                .pc_line_mapping
                .get(&(pc.as_u64() as usize))
                .copied()
            {
                let l = if line == 0 {
                    assembly.assembly_code.lines().next().unwrap()
                } else {
                    assembly.assembly_code.lines().skip(line).next().unwrap()
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
        _state: VmLocalStateData<'_, N, E>,
        _data: AfterDecodingData<N, E>,
        _memory: &Self::SupportedMemory,
    ) {
        if get_tracing_mode() != VmTracingOptions::ManualVerbose {
            return;
        }
    }
    fn before_execution(
        &mut self,
        state: VmLocalStateData<'_, N, E>,
        data: BeforeExecutionData<N, E>,
        memory: &Self::SupportedMemory,
    ) {
        if get_tracing_mode() != VmTracingOptions::ManualVerbose {
            return;
        }

        use zk_evm::zkevm_opcode_defs::*;

        match data.opcode.variant.opcode {
            Opcode::Ret(inner_variant) => {
                if !state
                    .vm_local_state
                    .callstack
                    .get_current_stack()
                    .is_local_frame
                {
                    // catch returndata
                    if inner_variant == RetOpcode::Ok || inner_variant == RetOpcode::Revert {
                        let src0 = data.src0_value;

                        let mut abi = RetABI::from_u256(src0.value);
                        match abi.page_forwarding_mode {
                            RetForwardPageType::ForwardFatPointer => {}
                            RetForwardPageType::UseHeap => {
                                abi.memory_quasi_fat_pointer.memory_page =
                                    CallStackEntry::<N, E>::heap_page_from_base(
                                        state
                                            .vm_local_state
                                            .callstack
                                            .get_current_stack()
                                            .base_memory_page,
                                    )
                                    .0;
                            }
                            RetForwardPageType::UseAuxHeap => {
                                abi.memory_quasi_fat_pointer.memory_page =
                                    CallStackEntry::<N, E>::aux_heap_page_from_base(
                                        state
                                            .vm_local_state
                                            .callstack
                                            .get_current_stack()
                                            .base_memory_page,
                                    )
                                    .0;
                            }
                        };

                        let returndata =
                            crate::runners::compiler_tests::dump_memory_page_using_fat_pointer(
                                memory,
                                abi.memory_quasi_fat_pointer,
                            );

                        println!(
                            "Performed return/revert with {} bytes with 0x{}",
                            returndata.len(),
                            hex::encode(&returndata)
                        );
                    } else {
                        println!("Returned with PANIC");
                    }
                }
            }
            Opcode::FarCall(_) => {
                // catch calldata
                let src0 = data.src0_value;
                let src1 = data.src1_value;
                let dest = u256_to_address_unchecked(&src1.value);

                let mut abi = FarCallABI::from_u256(src0.value);
                match abi.forwarding_mode {
                    FarCallForwardPageType::ForwardFatPointer => {}
                    FarCallForwardPageType::UseHeap => {
                        abi.memory_quasi_fat_pointer.memory_page =
                            CallStackEntry::<N, E>::heap_page_from_base(
                                state
                                    .vm_local_state
                                    .callstack
                                    .get_current_stack()
                                    .base_memory_page,
                            )
                            .0;
                    }
                    FarCallForwardPageType::UseAuxHeap => {
                        abi.memory_quasi_fat_pointer.memory_page =
                            CallStackEntry::<N, E>::aux_heap_page_from_base(
                                state
                                    .vm_local_state
                                    .callstack
                                    .get_current_stack()
                                    .base_memory_page,
                            )
                            .0;
                    }
                }

                let calldata = crate::runners::compiler_tests::dump_memory_page_using_fat_pointer(
                    memory,
                    abi.memory_quasi_fat_pointer,
                );

                println!(
                    "Performed far_call to {:?} with {} bytes with 0x{}",
                    dest,
                    calldata.len(),
                    hex::encode(&calldata)
                );
            }
            _ => {}
        }
    }
    fn after_execution(
        &mut self,
        state: VmLocalStateData<'_, N, E>,
        _data: AfterExecutionData<N, E>,
        _memory: &Self::SupportedMemory,
    ) {
        self.current_code_address = state
            .vm_local_state
            .callstack
            .get_current_stack()
            .code_address;
        if get_tracing_mode() != VmTracingOptions::ManualVerbose {
            return;
        }
        println!(
            "Registers: {:?}",
            state
                .vm_local_state
                .registers
                .iter()
                .map(|el| format!("0x{:064x}", el.value))
                .collect::<Vec<_>>()
        );
    }
}
