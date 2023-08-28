use super::*;

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use zk_evm::tracing::*;
use zk_evm::zkevm_opcode_defs::decoding::{AllowedPcOrImm, EncodingModeProduction, VmEncodingMode};
use zk_evm::zkevm_opcode_defs::{FatPointer, Opcode, REGISTERS_COUNT};

use crate::runners::compiler_tests::VmTracingOptions;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ContractSourceDebugInfo {
    pub assembly_code: String,
    pub pc_line_mapping: HashMap<usize, usize>,
    pub active_lines: HashSet<usize>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VmExecutionStep {
    pub contract_address: String,
    pub registers: [String; REGISTERS_COUNT],
    pub pc: u32,
    pub sp: u32,
    pub set_flags: Vec<String>,
    pub skip_cycle: bool,
    pub code_page_index: u32,
    pub heap_page_index: u32,
    pub stack_page_index: u32,
    pub register_interactions: HashMap<usize, MemoryAccessType>,
    pub memory_interactions: Vec<MemoryInteraction>,
    pub memory_snapshots: Vec<MemorySnapshot>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MemorySnapshot {
    memory_type: MemoryType,
    page: usize,
    length: usize,
    values: Vec<String>,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum MemoryType {
    heap,
    aux_heap,
    stack,
    fat_ptr,
    code,
    calldata,
    returndata,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum MemoryAccessType {
    Read,
    Write,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MemoryInteraction {
    pub memory_type: MemoryType,
    pub page: u32,
    pub address: u32,
    pub value: String,
    pub direction: MemoryAccessType,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VmTrace {
    pub steps: Vec<VmExecutionStep>,
    pub sources: HashMap<String, ContractSourceDebugInfo>,
}

use crate::default_environment::*;
use crate::runners::compiler_tests::calldata_to_aligned_data;

pub fn run_text_assembly_full_trace(
    assembly: String,
    calldata: Vec<u8>,
    num_cycles: usize,
) -> anyhow::Result<VmTrace> {
    let mut vm_assembly =
        Assembly::try_from(assembly.clone()).expect("must get a valid assembly as the input");

    let empty_callstack_dummy_debug_info = ContractSourceDebugInfo {
        assembly_code: "nop r0, r0, r0, r0".to_owned(),
        pc_line_mapping: HashMap::from([(0, 0)]),
        active_lines: HashSet::from([0]),
    };

    let mut context = VmExecutionContext::default();
    context.this_address = default_callee_address();

    // let debug_info = ContractSourceDebugInfo {
    //     assembly_code: vm_assembly.assembly_code.clone(),
    //     pc_line_mapping: vm_assembly.pc_line_mapping.clone(),
    //     active_lines: HashSet::new(),
    // };

    let assembly = vm_assembly.compile_to_bytecode().unwrap();

    // let mut tools = create_default_testing_tools();
    let mut tools = crate::runners::compiler_tests::create_default_testing_tools();

    let initial_bytecode_as_memory = contract_bytecode_to_words(&assembly);
    let aligned_calldata = calldata_to_aligned_data(&calldata);

    tools.memory.populate(vec![
        (CALLDATA_PAGE, aligned_calldata.clone()),
        (ENTRY_POINT_PAGE, initial_bytecode_as_memory.clone()),
    ]);

    let block_properties = create_default_block_properties();
    // let mut vm = create_vm_with_default_settings(&mut tools, &block_properties);

    let mut known_contracts = HashMap::new();
    known_contracts.insert(default_callee_address(), vm_assembly.clone());

    let (mut vm, _) = crate::runners::compiler_tests::create_vm::<false, 8, EncodingModeProduction>(
        &mut tools,
        &block_properties,
        context,
        &known_contracts,
        HashMap::new(),
        0,
    );

    use zk_evm::contract_bytecode_to_words;

    // set registers r1-r4 for external call convension
    vm.local_state.registers[0] =
        crate::utils::form_initial_calldata_ptr(CALLDATA_PAGE, calldata.len() as u32);
    vm.local_state.registers[1] = PrimitiveValue::empty();
    vm.local_state.registers[2] = PrimitiveValue::empty();
    vm.local_state.registers[3] = PrimitiveValue::empty();

    let mut tracer = VmDebugTracer::new_from_entry_point(default_callee_address(), &vm_assembly);
    tracer
        .debug_info
        .insert(Address::default(), empty_callstack_dummy_debug_info);

    for _ in 0..num_cycles {
        vm.cycle(&mut tracer)?;
    }

    let VmDebugTracer {
        steps, debug_info, ..
    } = tracer;

    let mut sources = HashMap::new();
    for (k, v) in debug_info.into_iter() {
        let k = format!("{:?}", k);
        sources.insert(k, v);
    }

    let full_trace = VmTrace { steps, sources };

    Ok(full_trace)
}

fn error_flags_into_description(flags: &ErrorFlags) -> Vec<String> {
    if flags.is_empty() {
        return vec![];
    }

    let mut result = vec![];
    if flags.contains(ErrorFlags::NOT_ENOUGH_ERGS) {
        result.push("Not enough ergs".to_owned());
    }
    if flags.contains(ErrorFlags::CALLSTACK_IS_FULL) {
        result.push("Callstack is full".to_owned());
    }

    result
}

use zk_evm::flags::*;

fn flags_into_description(flags: &Flags) -> Vec<String> {
    let mut result = vec![];
    if flags.overflow_or_less_than_flag {
        result.push("lt".to_owned());
    }
    if flags.equality_flag {
        result.push("eq".to_owned());
    }
    if flags.greater_than_flag {
        result.push("gt".to_owned());
    }

    result
}

pub struct VmDebugTracer<const N: usize = 8, E: VmEncodingMode<N> = EncodingModeProduction> {
    pub debug_info: HashMap<Address, ContractSourceDebugInfo>,
    regs_before: Option<[PrimitiveValue; REGISTERS_COUNT]>,
    aux_info: Option<AfterDecodingData<N, E>>,
    callstack_info: Option<CallStackEntry<N, E>>,
    cycle_number: u32,
    did_call_recently: bool,
    did_return_recently: bool,
    pub steps: Vec<VmExecutionStep>,
}

impl<const N: usize, E: VmEncodingMode<N>> std::fmt::Debug for VmDebugTracer<N, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VmDebugTracer").finish()
    }
}

impl<const N: usize, E: VmEncodingMode<N>> VmDebugTracer<N, E> {
    pub fn new_from_entry_point(entry_address: Address, source: &Assembly) -> Self {
        let debug_info = ContractSourceDebugInfo {
            assembly_code: source.assembly_code.clone(),
            pc_line_mapping: source.pc_line_mapping.clone(),
            active_lines: HashSet::new(),
        };

        let mut initial_info = HashMap::new();
        initial_info.insert(entry_address, debug_info);

        Self {
            debug_info: initial_info,
            regs_before: None,
            aux_info: None,
            callstack_info: None,
            did_call_recently: true,
            did_return_recently: false,
            cycle_number: 0u32,
            steps: vec![],
        }
    }
    pub fn add_known_contracts(&mut self, other_contracts: &HashMap<Address, Assembly>) {
        self.debug_info
            .extend(other_contracts.clone().into_iter().map(|(k, v)| {
                let info = ContractSourceDebugInfo {
                    assembly_code: v.assembly_code.clone(),
                    pc_line_mapping: v.pc_line_mapping.clone(),
                    active_lines: HashSet::new(),
                };

                (k, info)
            }));
    }
}

use crate::runners::hashmap_based_memory::SimpleHashmapMemory;

impl<const N: usize, E: VmEncodingMode<N>> zk_evm::tracing::Tracer<N, E> for VmDebugTracer<N, E> {
    const CALL_BEFORE_DECODING: bool = false;
    const CALL_AFTER_DECODING: bool = true;
    const CALL_BEFORE_EXECUTION: bool = true;
    const CALL_AFTER_EXECUTION: bool = true;

    type SupportedMemory = SimpleHashmapMemory;

    fn before_decoding(
        &mut self,
        _state: VmLocalStateData<'_, N, E>,
        _memory: &Self::SupportedMemory,
    ) {
    }
    fn after_decoding(
        &mut self,
        state: VmLocalStateData<'_, N, E>,
        data: AfterDecodingData<N, E>,
        memory: &Self::SupportedMemory,
    ) {
        debug_assert!(self.aux_info.is_none());
        debug_assert!(self.regs_before.is_none());

        // we need to know
        // - register reads
        // - memory reads

        let skip_cycle = data.did_skip_cycle;
        let errors = error_flags_into_description(&data.error_flags_accumulated);
        let current_context = state.vm_local_state.callstack.get_current_stack();
        let current_pc = current_context.pc;
        let current_sp = current_context.sp;
        let code_address = current_context.code_address;
        let contract_address = format!("0x{:x}", current_context.this_address);
        let code_page = current_context.code_page.0;
        let base_memory_page = current_context.base_memory_page.0;
        self.callstack_info = Some(current_context.clone());
        drop(current_context);
        if let Some(info) = self.debug_info.get_mut(&code_address) {
            info.active_lines.insert(current_pc.as_u64() as usize);
        }
        // self.debug_info.active_lines.insert(current_pc as usize);
        let flags = flags_into_description(&state.vm_local_state.flags);

        self.regs_before = Some(state.vm_local_state.registers);

        let registers = state
            .vm_local_state
            .registers
            .map(|el| format!("0x{:x}", el.value));
        // .map(|el| format!("0x{:064x}", el));

        let error = if let Some(e) = errors.first() {
            Some(e.clone())
        } else {
            None
        };
        let mut trace_step = VmExecutionStep {
            contract_address,
            registers,
            pc: current_pc.as_u64() as u32,
            sp: current_sp.as_u64() as u32,
            set_flags: flags,
            skip_cycle,
            code_page_index: code_page,
            heap_page_index: CallStackEntry::<N, E>::heap_page_from_base(MemoryPage(
                base_memory_page,
            ))
            .0,
            stack_page_index: CallStackEntry::<N, E>::stack_page_from_base(MemoryPage(
                base_memory_page,
            ))
            .0,
            register_interactions: HashMap::new(),
            memory_interactions: vec![],
            memory_snapshots: vec![],
            error,
        };

        // special case for initial cycle
        if self.did_call_recently {
            let (calldata_page, range) =
                crate::runners::compiler_tests::fat_ptr_into_page_and_aligned_words_range(
                    state.vm_local_state.registers[0],
                );

            let initial_calldata = memory.dump_page_content(calldata_page, range);
            let len_words = initial_calldata.len();

            let initial_calldata = initial_calldata
                .into_iter()
                .map(|el| format!("0x{}", hex::encode(&el)))
                .collect();
            let snapshot = MemorySnapshot {
                memory_type: MemoryType::calldata,
                page: calldata_page as usize,
                length: len_words as usize,
                values: initial_calldata,
            };

            trace_step.memory_snapshots.push(snapshot);
            self.did_call_recently = false;
        }

        if self.did_return_recently {
            let (returndata_page, range) =
                crate::runners::compiler_tests::fat_ptr_into_page_and_aligned_words_range(
                    state.vm_local_state.registers[0],
                );

            let mut fat_ptr = FatPointer::from_u256(state.vm_local_state.registers[0].value);
            if state.vm_local_state.registers[0].is_pointer == false {
                fat_ptr = FatPointer::empty();
            }
            let returndata_len = fat_ptr.length - fat_ptr.offset;
            let initial_returndata = memory.dump_page_content(returndata_page, range);
            let initial_returndata = initial_returndata
                .into_iter()
                .map(|el| format!("0x{}", hex::encode(&el)))
                .collect();

            let snapshot = MemorySnapshot {
                memory_type: MemoryType::returndata,
                page: returndata_page as usize,
                length: returndata_len as usize,
                values: initial_returndata,
            };

            trace_step.memory_snapshots.push(snapshot);

            self.did_return_recently = false;
        }

        self.steps.push(trace_step);
    }
    fn before_execution(
        &mut self,
        state: VmLocalStateData<'_, N, E>,
        data: BeforeExecutionData<N, E>,
        memory: &Self::SupportedMemory,
    ) {
        let current_context = state.vm_local_state.callstack.get_current_stack();
        let base_memory_page = current_context.base_memory_page.0;
        let code_page = current_context.code_page.0;

        let trace_step = self.steps.last_mut().unwrap();

        if let Some(mem) = data.src0_mem_location {
            let MemoryLocation {
                memory_type,
                page,
                index,
            } = mem;
            let page = page.0;
            let index = index.0;
            let mem_interaction = match page {
                page if page
                    == CallStackEntry::<N, E>::heap_page_from_base(MemoryPage(
                        base_memory_page,
                    ))
                    .0
                    || page
                        == CallStackEntry::<N, E>::stack_page_from_base(MemoryPage(
                            base_memory_page,
                        ))
                        .0
                    || page == code_page =>
                {
                    let memory_type = if page
                        == CallStackEntry::<N, E>::heap_page_from_base(MemoryPage(base_memory_page))
                            .0
                    {
                        assert_eq!(memory_type, zk_evm::abstractions::MemoryType::Heap);
                        MemoryType::heap
                    } else if page
                        == CallStackEntry::<N, E>::stack_page_from_base(MemoryPage(
                            base_memory_page,
                        ))
                        .0
                    {
                        assert_eq!(memory_type, zk_evm::abstractions::MemoryType::Stack);
                        MemoryType::stack
                    } else if page == code_page {
                        assert_eq!(memory_type, zk_evm::abstractions::MemoryType::Code);
                        MemoryType::code
                    } else {
                        unreachable!()
                    };

                    // let value = memory
                    //     .inner
                    //     .get(&page)
                    //     .unwrap_or(&vec![])
                    //     .get(index as usize)
                    //     .copied()
                    //     .unwrap_or(U256::zero());

                    let value = memory
                        .inner
                        .get(&page)
                        .unwrap_or(&HashMap::new())
                        .get(&index)
                        .map(|el| el.value)
                        .unwrap_or(U256::zero());

                    let value = format!("0x{:064x}", value);
                    let mem_interaction = MemoryInteraction {
                        memory_type,
                        page,
                        address: index,
                        value,
                        direction: MemoryAccessType::Read,
                    };

                    mem_interaction
                }
                // page if page == calldata_page ||
                // page == returndata_page => {
                //     let (memory_type, offset, len) = if page == calldata_page {
                //         (MemoryType::calldata, calldata_offset, calldata_len)
                //     } else if page == returndata_page {
                //         (MemoryType::returndata, returndata_offset, returndata_len)
                //     } else {
                //         unreachable!()
                //     };

                //     let value = main.memory.inner.get(&page).unwrap_or(&vec![]).get(index as usize).copied().unwrap_or(U256::zero());
                //     let value = format!("0x{:x}", value);

                //     let (index, of) = index.overflowing_sub(offset);
                //     assert!(!of);
                //     assert!(index < len);
                //     let mem_interaction = MemoryInteraction {
                //         memory_type,
                //         page,
                //         address: index,
                //         value,
                //         direction: MemoryAccessType::Read
                //     };

                //     mem_interaction
                // }
                _ => {
                    unreachable!()
                }
            };

            trace_step.memory_interactions.push(mem_interaction);
        } else {
            let src0_reg_idx = data.opcode.src0_reg_idx;
            if src0_reg_idx != 0 {
                trace_step
                    .register_interactions
                    .insert(src0_reg_idx as usize, MemoryAccessType::Read);
            }
        }
        let src1_reg_idx = data.opcode.src1_reg_idx;
        if src1_reg_idx != 0 {
            trace_step
                .register_interactions
                .insert(src1_reg_idx as usize, MemoryAccessType::Read);
        }
    }
    fn after_execution(
        &mut self,
        state: VmLocalStateData<'_, N, E>,
        data: AfterExecutionData<N, E>,
        memory: &Self::SupportedMemory,
    ) {
        // let aux = self.aux_info.take().unwrap();
        let _regs_before = self.regs_before.take().unwrap();
        let potentially_previous_context = self.callstack_info.take().unwrap();
        let code_page = potentially_previous_context.code_page.0;
        let base_memory_page = potentially_previous_context.base_memory_page.0;
        // - register writes
        // - memory writes

        let trace_step = self.steps.last_mut().unwrap();
        trace_step.registers = state
            .vm_local_state
            .registers
            .map(|el| format!("0x{:x}", el.value));
        // .map(|el| format!("0x{:064x}", el));

        if let Some(mem) = data.dst0_mem_location {
            let MemoryLocation {
                memory_type,
                page,
                index,
            } = mem;
            let page = page.0;
            let index = index.0;
            let mem_interaction = match page {
                page if page
                    == CallStackEntry::<N, E>::heap_page_from_base(MemoryPage(
                        base_memory_page,
                    ))
                    .0
                    || page
                        == CallStackEntry::<N, E>::stack_page_from_base(MemoryPage(
                            base_memory_page,
                        ))
                        .0
                    || page == code_page =>
                {
                    let memory_type = if page
                        == CallStackEntry::<N, E>::heap_page_from_base(MemoryPage(base_memory_page))
                            .0
                    {
                        assert_eq!(memory_type, zk_evm::abstractions::MemoryType::Heap);
                        MemoryType::heap
                    } else if page
                        == CallStackEntry::<N, E>::stack_page_from_base(MemoryPage(
                            base_memory_page,
                        ))
                        .0
                    {
                        assert_eq!(memory_type, zk_evm::abstractions::MemoryType::Stack);
                        MemoryType::stack
                    } else if page == code_page {
                        assert_eq!(memory_type, zk_evm::abstractions::MemoryType::Code);
                        MemoryType::code
                    } else {
                        unreachable!()
                    };

                    // let value = memory
                    //     .inner
                    //     .get(&page)
                    //     .unwrap_or(&vec![])
                    //     .get(index as usize)
                    //     .copied()
                    //     .unwrap_or(U256::zero());

                    let value = memory
                        .inner
                        .get(&page)
                        .unwrap_or(&HashMap::new())
                        .get(&index)
                        .map(|el| el.value)
                        .unwrap_or(U256::zero());

                    let value = format!("0x{:064x}", value);
                    let mem_interaction = MemoryInteraction {
                        memory_type,
                        page,
                        address: index,
                        value,
                        direction: MemoryAccessType::Write,
                    };

                    mem_interaction
                }
                // page if page == calldata_page ||
                // page == returndata_page => {
                //     let (memory_type, offset, len) = if page == calldata_page {
                //         (MemoryType::calldata, calldata_offset, calldata_len)
                //     } else if page == returndata_page {
                //         (MemoryType::returndata, returndata_offset, returndata_len)
                //     } else {
                //         unreachable!()
                //     };

                //     let value = main.memory.inner.get(&page).unwrap_or(&vec![]).get(index as usize).copied().unwrap_or(U256::zero());
                //     let value = format!("0x{:x}", value);

                //     let (index, of) = index.overflowing_sub(offset);
                //     assert!(!of);
                //     assert!(index < len);
                //     let mem_interaction = MemoryInteraction {
                //         memory_type,
                //         page,
                //         address: index,
                //         value,
                //         direction: MemoryAccessType::Write
                //     };

                //     mem_interaction
                // }
                _ => {
                    unreachable!()
                }
            };

            trace_step.memory_interactions.push(mem_interaction);
        } else {
            let dst0_reg_idx = data.opcode.dst0_reg_idx;
            if dst0_reg_idx != 0 {
                trace_step
                    .register_interactions
                    .insert(dst0_reg_idx as usize, MemoryAccessType::Write);
            }
        }
        let dst1_reg_idx = data.opcode.dst1_reg_idx;
        if dst1_reg_idx != 0 {
            trace_step
                .register_interactions
                .insert(dst1_reg_idx as usize, MemoryAccessType::Write);
        }

        // special case for call or return
        if let Opcode::FarCall(_far_call_variant) = data.opcode.variant.opcode {
            self.did_call_recently = true;
        }

        // special case for call or return
        if let Opcode::Ret(_return_variant) = data.opcode.variant.opcode {
            if !potentially_previous_context.is_local_frame {
                // only on far return
                self.did_return_recently = true;
            }
        }

        self.cycle_number += 1;
    }
}

use crate::runners::compiler_tests::VmLaunchOption;

pub(crate) fn run_inner(calldata: &[u8], options: VmLaunchOption, assembly_text: &str) {
    use crate::runners::compiler_tests::*;

    let assembly = Assembly::try_from(assembly_text.to_owned()).unwrap();
    let bytecode = assembly.clone().compile_to_bytecode().unwrap();
    let hash = U256::from(zk_evm::utils::bytecode_to_code_hash(&bytecode).unwrap());
    let mut known_contracts = HashMap::new();
    known_contracts.insert(hash, assembly.clone());
    let snapshot = run_vm(
        "manual".to_owned(),
        assembly.clone(),
        calldata,
        HashMap::new(),
        None,
        options,
        u16::MAX as usize,
        known_contracts,
        U256::zero(),
    )
    .unwrap();

    let VmSnapshot {
        registers,

        execution_has_ended,

        storage,

        execution_result,
        returndata_bytes,
        events,

        serialized_events,
        ..
    } = snapshot;
    dbg!(execution_has_ended);
    dbg!(execution_result);
    dbg!(registers);
    dbg!(hex::encode(&returndata_bytes));
    dbg!(events);
    dbg!(storage);
    println!("{}", serialized_events);
}

use crate::runners::compiler_tests::VmExecutionContext;

pub(crate) fn run_inner_with_context(
    calldata: &[u8],
    options: VmLaunchOption,
    assembly_text: &str,
    context: VmExecutionContext,
) {
    use crate::runners::compiler_tests::*;

    let assembly = Assembly::try_from(assembly_text.to_owned()).unwrap();
    let bytecode = assembly.clone().compile_to_bytecode().unwrap();
    let hash = U256::from(zk_evm::utils::bytecode_to_code_hash(&bytecode).unwrap());
    let mut known_contracts = HashMap::new();
    known_contracts.insert(hash, assembly.clone());
    let entry_address = context.this_address;
    let mut contracts: HashMap<Address, Assembly> = HashMap::new();
    contracts.insert(entry_address, assembly.clone());
    let snapshot = run_vm_multi_contracts(
        "manual".to_owned(),
        contracts,
        calldata,
        HashMap::new(),
        entry_address,
        Some(context),
        options,
        u16::MAX as usize,
        known_contracts,
        U256::zero(),
    )
    .unwrap();

    let VmSnapshot {
        registers,

        execution_has_ended,

        storage,

        execution_result,
        returndata_bytes,
        events,
        ..
    } = snapshot;
    dbg!(execution_has_ended);
    dbg!(execution_result);
    dbg!(registers);
    dbg!(hex::encode(&returndata_bytes));
    dbg!(events);
    dbg!(storage);
}

#[cfg(any(test, feature = "external_testing"))]
pub mod test {
    use crate::runners::compiler_tests::{set_tracing_mode, VmExecutionContext};

    use super::*;

    const SIMPLE_ASSEMBLY: &'static str = r#"
    .text
    main:
        context.this r2
        far_call r2, r0, @panic
    ret_ok:
        ret.ok r1
    panic:
        ret.panic r0
    "#;

    #[test]
    fn run_something() {
        use super::*;

        // set_tracing_mode(VmTracingOptions::ManualVerbose);

        let mut input = vec![0u8; 64];
        input[31] = 1;
        input[63] = 2;

        let trace = run_text_assembly_full_trace(SIMPLE_ASSEMBLY.to_owned(), input, 1000).unwrap();

        let _ = std::fs::remove_file("tmp.json");
        let mut file = std::fs::File::create("tmp.json").unwrap();
        let json = serde_json::to_string(&trace).unwrap();

        use std::io::Write;
        file.write_all(json.as_bytes()).unwrap();
    }

    #[test]
    fn test_manually() {
        run_inner(
            &hex::decode("5a8ac02d").unwrap(),
            VmLaunchOption::Default,
            SIMPLE_ASSEMBLY,
        );
    }

    #[test]
    fn test_constructor_manually() {
        run_inner(
            &hex::decode("5a8ac02d").unwrap(),
            VmLaunchOption::Constructor,
            SIMPLE_ASSEMBLY,
        );
    }

    const WITH_EVENTS_ASSEMBLY: &'static str = r#"
    .text
	.file	"Test_41"
	.rodata.cst32
	.p2align	5
CPI0_0:
	.cell 16777184
CPI0_1:
	.cell 16777152
	.text
	.globl	__entry
__entry:
.func_begin0:
	nop	stack+=[7]
	add	@CPI0_0[0], r0, r4
	uma.heap_write	r4, r1, r0
	add	@CPI0_1[0], r0, r1
	uma.heap_write	r1, r2, r0
	and	1, r3, r2
	add	0, r0, r1
	sub!	r2, r1, r2
	jump.ne	@.BB0_3
	jump	@.BB0_4
.BB0_3:
	add	128, r0, r2
	add	64, r0, r3
	uma.heap_write	r3, r2, r0
	add	@CPI0_0[0], r0, r3
	uma.heap_write	r3, r2, r0
	add	@CPI0_1[0], r0, r2
	uma.heap_write	r2, r1, r0
	jump	@.BB0_2
.BB0_4:
.tmp0:
	near_call	r0, @__selector, @.BB0_1
.tmp1:
.BB0_2:
	add	@CPI0_0[0], r0, r1
	uma.heap_read	r1, r0, r1
	add	@CPI0_1[0], r0, r2
	uma.heap_read	r2, r0, r2
	shl.s	32, r2, r2
	add	r2, r1, r1
	nop	stack-=[7]
	ret
.BB0_1:
.tmp2:
	add	96, r0, r1
	uma.heap_read	r1, r0, r1
	add	1, r0, r2
	sub!	r1, r2, r1
	jump.eq	@.BB0_2
	jump	@.BB0_5
.BB0_5:
	ret.panic r0
.func_end0:

	.rodata.cst32
	.p2align	5
CPI1_0:
	.cell 16777152
CPI1_1:
	.cell -4
CPI1_2:
	.cell 16777184
CPI1_3:
	.cell -26959946667150639794667015087019630673637144422540572481103610249216
CPI1_4:
	.cell 18957599724396051841795879574321980412497429171365101850932991142745373409280
CPI1_5:
	.cell -57896044618658097711785492504343953926634992332820282019728792003956564819968
CPI1_6:
	.cell 52549307936116447935400929610646957733401880485183426834406134051980385742268
CPI1_7:
	.cell 3735928559
CPI1_8:
	.cell 4588150944598771411443733190509957345959890284296518184527857552952610963977
CPI1_9:
	.cell 274877906945
CPI1_10:
	.cell 12648430
CPI1_11:
	.cell 25316831761693835374013077841922713676043321329999779885431350772770455861368
CPI1_12:
	.cell 137438953474
	.text
__selector:
.func_begin1:
	add	128, r0, r2
	add	64, r0, r1
	uma.heap_write	r1, r2, r0
	add	@CPI1_0[0], r0, r3
	uma.heap_read	r3, r0, r3
	add	3, r0, r4
	sub!	r3, r4, r4
	jump.gt	@.BB1_2
	jump	@.BB1_1
.BB1_2:
	add	@CPI1_1[0], r0, r5
	add	@CPI1_2[0], r0, r4
	uma.heap_read	r4, r0, r4
	uma.calldata_read	r4, r0, r6
	and	@CPI1_3[0], r6, r6
	add	@CPI1_4[0], r0, r7
	sub!	r6, r7, r6
	add	r3, r5, r3
	add	32, r0, r5
	add	@CPI1_5[0], r0, r6
	sub!	r3, r5, r5
	add	0, r0, r5
	add.lt	r6, r0, r5
	and	r3, r6, r7
	add	0, r0, r3
	sub!	r7, r3, r8
	add	0, r0, r8
	add.gt	r6, r0, r8
	sub!	r7, r6, r6
	add.eq	r8, r0, r5
	sub!	r5, r3, r5
	add	4, r4, r4
	uma.calldata_read	r4, r0, r4
	add	@CPI1_6[0], r0, r5
	event.first	1, r5
	uma.heap_write	r2, r4, r0
	add	@CPI1_7[0], r0, r5
	add	160, r0, r6
	uma.heap_write	r6, r5, r0
	add	@CPI1_8[0], r0, r5
	add	@CPI1_9[0], r0, r7
	event.first	r7, r5
	uma.heap_read	r6, r0, r5
	uma.heap_read	r2, r0, r2
	event	r2, r5
	add	@CPI1_10[0], r0, r2
	uma.heap_read	r1, r0, r5
	uma.heap_write	r5, r2, r0
	add	@CPI1_11[0], r0, r5
	add	@CPI1_12[0], r0, r6
	event.first	r6, r5
	event	r4, r2
	uma.heap_read	r1, r0, r1
	add	@CPI1_2[0], r0, r2
	uma.heap_write	r2, r1, r0
	add	@CPI1_0[0], r0, r1
	uma.heap_write	r1, r3, r0
	ret
.BB1_1:
	add	0, r0, r1
	add	@CPI1_2[0], r0, r2
	uma.heap_write	r2, r1, r0
	add	@CPI1_0[0], r0, r2
	uma.heap_write	r2, r1, r0
	ret.panic r0
.func_end1:

	.note.GNU-stack
    "#;

    #[test]
    fn run_for_events() {
        run_inner(
            &hex::decode("29e99f07000000000000000000000000000000000000000000000000000000000000002a")
                .unwrap(),
            VmLaunchOption::Default,
            WITH_EVENTS_ASSEMBLY,
        );
    }

    const SIMPLE_TOUCH_STORAGE: &'static str = r#"
    .text
	.file	"Test_41"
	.rodata.cst32
	.p2align	5
CPI0_0:
	.cell 16777184
CPI0_1:
	.cell 16777152
	.text
	.globl	__entry
__entry:
.func_begin0:
    add 42, r0, r1
    add 1, r0, r2
    sstore r2, r1
    ret.ok r0

	.note.GNU-stack
    "#;

    #[test]
    fn run_for_simple_storage_touch() {
        run_inner(
            &hex::decode("").unwrap(),
            VmLaunchOption::Default,
            SIMPLE_TOUCH_STORAGE,
        );
    }

    const SIMPLE_STORAGE_WITH_ROLLBACK: &'static str = r#"
    .text
	.file	"Test_41"
	.rodata.cst32
	.p2align	5
CPI0_0:
	.cell 16777184
CPI0_1:
	.cell 16777152
	.text
	.globl	__entry
__entry:
.func_begin0:
    add 42, r0, r1
    add 1, r0, r2
    sstore r2, r1
    ret.revert r0

	.note.GNU-stack
    "#;

    #[test]
    fn run_for_simple_storage_with_rollback() {
        run_inner(
            &hex::decode("").unwrap(),
            VmLaunchOption::Default,
            SIMPLE_STORAGE_WITH_ROLLBACK,
        );
    }

    const SIMPLE_STORAGE_WITH_ROLLBACK_OF_CHILD: &'static str = r#"
    .text
	.file	"Test_41"
	.rodata.cst32
	.p2align	5
CPI0_0:
	.cell 16777184
CPI0_1:
	.cell 16777152
	.text
	.globl	__entry
__entry:
.func_begin0:
    add 42, r0, r1
    add 1, r0, r2
    sstore r2, r1
    call @.child
    ret.revert r0
.child:
    add 99, r0, r1
    add 2, r0, r2
    sstore r2, r1
    add 0, r0, r1
    ret.ok r1
.note.GNU-stack
    "#;

    #[test]
    fn run_for_simple_storage_with_rollback_in_inner_frame() {
        run_inner(
            &hex::decode("").unwrap(),
            VmLaunchOption::Default,
            SIMPLE_STORAGE_WITH_ROLLBACK_OF_CHILD,
        );
    }

    const MANUAL_DEFAULT_UNWIND_LABEL_ACCESS: &'static str = r#"
                .text
                .file    "Test_16"
                .globl    __entry
            __entry:
            .func_begin0:
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
                add    r2, r1, r1
                add    0, r0, r2
                add    r2, r0, r3
                near_call    r0, @__cxa_throw, @DEFAULT_UNWIND
            .func_end0:

            __selector:
            .func_begin1:
                nop    stack+=[4]
                add    128, r0, r1
                add    64, r0, r2
                add    r2, r0, stack-[3]
                uma.heap_write    r2, r1, r0
                add    @CPI1_0[0], r0, r1
                uma.heap_read    r1, r0, r2
                add    3, r0, r1
                sub!    r2, r1, r1
                jump.le    @.BB1_1
                add    @CPI1_1[0], r0, r3
                add    @CPI1_2[0], r0, r1
                uma.heap_read    r1, r0, r1
                uma.calldata_read    r1, r0, r4
                and    @CPI1_3[0], r4, r4
                add    @CPI1_4[0], r0, r5
                sub!    r4, r5, r4
                jump.ne    @.BB1_1
                add    r2, r3, r2
                add    @CPI1_5[0], r0, r3
                add    stack-[4], 0, r4
                sub!    r2, r4, r4
                add    0, r0, r4
                add.lt    r3, r0, r4
                and    r2, r3, r2
                add    0, r0, r5
                sub!    r2, r5, r6
                add    0, r0, r6
                add.gt    r3, r0, r6
                sub!    r2, r3, r2
                add    r4, r0, r2
                add.eq    r6, r0, r2
                sub!    r2, r5, r2
                jump.ne    @.BB1_1
                add    4, r1, r1
                uma.calldata_read    r1, r0, r2
            .tmp3:
                add    0, r0, r1
                add    r1, r0, stack-[1]
                add    r2, r0, stack-[2]
                near_call    r0, @__signextend, @.BB1_6
            .tmp4:
                add    stack-[3], 0, r2
                sub!    r2, r1, r1
                jump.ne    @.BB1_9
                add    @CPI1_2[0], r0, r1
                uma.heap_read    r1, r0, r1
                add    36, r1, r1
                uma.calldata_read    r1, r0, r2
            .tmp10:
                add    0, r0, r1
                add    r1, r0, stack-[0]
                add    r2, r0, stack-[1]
                near_call    r0, @__signextend, @.BB1_12
            .tmp11:
                add    stack-[2], 0, r2
                sub!    r2, r1, r1
                add    stack-[3], 0, r2
                jump.ne    @.BB1_15
            .tmp17:
                add    0, r0, r1
                near_call    r0, @__signextend, @.BB1_2
                add    r1, r0, stack-[0]
            .tmp18:
            .tmp19:
                add    0, r0, r1
                add    r1, r0, stack-[2]
                add    stack-[2], 0, r2
                near_call    r0, @__signextend, @.BB1_2
            .tmp20:
                add    @CPI1_6[0], r0, r2
                add    stack-[1], 0, r8
                sub!    r8, r2, r2
                add    stack-[3], 0, r7
                jump.gt    @.BB1_23
                sub    127, r8, r2
                add    @CPI1_5[0], r0, r3
                sub!    r1, r2, r4
                add    0, r0, r4
                add.le    r3, r0, r4
                and    r2, r3, r2
                and    r1, r3, r5
                sub!    r5, r2, r6
                add    0, r0, r6
                add.gt    r3, r0, r6
                xor    r5, r2, r2
                sub!    r2, r3, r2
                add    r4, r0, r2
                add.eq    r6, r0, r2
                sub!    r2, r7, r2
                jump.eq    @.BB1_21
            .BB1_23:
                sub    @CPI1_8[0], r8, r2
                and    @CPI1_5[0], r1, r4
                and    r2, r3, r5
                sub!    r4, r5, r6
                add    0, r0, r6
                add.gt    r3, r0, r6
                xor    r4, r5, r4
                sub!    r1, r2, r2
                add    0, r0, r2
                add.lt    r3, r0, r2
                sub!    r4, r3, r3
                add.eq    r6, r0, r2
                sub!    r2, r7, r2
                add    0, r0, r2
                add.ne    1, r0, r2
                shr.s    255, r8, r3
                and    r2, r3, r2
                sub!    r2, r7, r2
                jump.ne    @.BB1_24
                add    r1, r8, r2
                add    stack-[4], 0, r1
                uma.heap_read    r1, r0, r1
                add    r1, r0, stack-[3]
            .tmp25:
                add    0, r0, r1
                near_call    r0, @__signextend, @.BB1_27
            .tmp26:
                add    stack-[4], 0, r2
                uma.heap_write    r2, r1, r0
                add    @CPI1_2[0], r0, r1
                uma.heap_write    r1, r2, r0
                add    32, r0, r1
                add    @CPI1_0[0], r0, r2
                uma.heap_write    r2, r1, r0
                nop    stack-=[4]
                ret
            .BB1_1:
                add    0, r0, r1
                add    @CPI1_2[0], r0, r2
                uma.heap_write    r2, r1, r0
                add    @CPI1_0[0], r0, r2
                uma.heap_write    r2, r1, r0
                add    r1, r0, r2
                add    r1, r0, r3
                near_call    r0, @__cxa_throw, @DEFAULT_UNWIND
            .BB1_9:
                add    @CPI1_2[0], r0, r1
                add    stack-[2], 0, r2
                uma.heap_write    r1, r2, r0
                add    @CPI1_0[0], r0, r1
                uma.heap_write    r1, r2, r0
            .tmp8:
                add    r2, r0, r1
                add    r2, r0, r3
                near_call    r0, @__cxa_throw, @.BB1_2
            .tmp9:
            .BB1_15:
                add    @CPI1_2[0], r0, r1
                add    stack-[1], 0, r2
                uma.heap_write    r1, r2, r0
                add    @CPI1_0[0], r0, r1
                uma.heap_write    r1, r2, r0
            .tmp15:
                add    r2, r0, r1
                add    r2, r0, r3
                near_call    r0, @__cxa_throw, @.BB1_2
            .tmp16:
            .BB1_24:
                add    @CPI1_7[0], r0, r1
                uma.heap_write    r7, r1, r0
                add    17, r0, r1
                add    4, r0, r2
                uma.heap_write    r2, r1, r0
                add    @CPI1_2[0], r0, r1
                uma.heap_write    r1, r7, r0
                add    36, r0, r1
                add    @CPI1_0[0], r0, r2
                uma.heap_write    r2, r1, r0
            .tmp23:
                add    r7, r0, r1
                add    r7, r0, r2
                add    r7, r0, r3
                near_call    r0, @__cxa_throw, @.BB1_2
            .tmp24:
            .BB1_21:
                add    @CPI1_7[0], r0, r1
                uma.heap_write    r7, r1, r0
                add    17, r0, r1
                add    4, r0, r2
                uma.heap_write    r2, r1, r0
                add    @CPI1_2[0], r0, r1
                uma.heap_write    r1, r7, r0
                add    36, r0, r1
                add    @CPI1_0[0], r0, r2
                uma.heap_write    r2, r1, r0
            .tmp21:
                add    r7, r0, r1
                add    r7, r0, r2
                add    r7, r0, r3
                near_call    r0, @__cxa_throw, @.BB1_2
            .tmp22:
            .BB1_27:
            .tmp27:
            .tmp28:
                add    0, r0, r1
                add    r1, r0, r2
                add    r1, r0, r3
                near_call    r0, @__cxa_throw, @.BB1_2
            .tmp29:
            .BB1_12:
            .tmp12:
            .tmp13:
                add    0, r0, r1
                add    r1, r0, r2
                add    r1, r0, r3
                near_call    r0, @__cxa_throw, @.BB1_2
            .tmp14:
            .BB1_6:
            .tmp5:
            .tmp6:
                add    0, r0, r1
                add    r1, r0, r2
                add    r1, r0, r3
                near_call    r0, @__cxa_throw, @.BB1_2
            .tmp7:
            .BB1_2:
            .tmp30:
                add    0, r0, r1
                add    r1, r0, r2
                add    r1, r0, r3
                near_call    r0, @__cxa_throw, @DEFAULT_UNWIND
            .func_end1:

            __signextend:
                shl.s    3, r1, r1
                or    7, r1, r3
                shl    @CPI2_0[0], r3, r4
                shr    r2, r3, r3
                and    1, r3, r3
                add    0, r0, r5
                sub!    r3, r5, r3
                add    0, r0, r3
                add.ne    r4, r0, r3
                sub    249, r1, r1
                shl    r2, r1, r2
                shr    r2, r1, r1
                or    r3, r1, r1
                ret

            __cxa_throw:
                revert

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
                .cell -37091031237102535244306088620066292187377964821292984992142541772018409275392
            CPI1_5:
                .cell -57896044618658097711785492504343953926634992332820282019728792003956564819968
            CPI1_6:
                .cell 57896044618658097711785492504343953926634992332820282019728792003956564819967
            CPI1_7:
                .cell 35408467139433450592217433187231851964531694900788300625387963629091585785856
            CPI1_8:
                .cell -128
            CPI2_0:
                .cell -1
    "#;

    #[test]
    fn run_parse_manual_default_unwind() {
        run_inner(
            &hex::decode("").unwrap(),
            VmLaunchOption::Default,
            MANUAL_DEFAULT_UNWIND_LABEL_ACCESS,
        );
    }

    const ENSURE_PROPER_RETURN_ON_REVERT: &'static str = r#"
            .text
            .file	"Test_73"
            .globl	__entry
        __entry:
        .func_begin0:
            add	r1, r0, r4
            add	@CPI0_0[0], r0, r1
            uma.heap_write	r1, r4, r0
            add	@CPI0_1[0], r0, r1
            uma.heap_write	r1, r2, r0
            and	1, r3, r1
            add	0, r0, r2
            sub!	r1, r2, r1
            jump.ne	@.BB0_4
            jump	@.BB0_5
        .BB0_2:
        .tmp4:
            add	@CPI0_2[0], r0, r1
            uma.heap_read	r1, r0, r1
            add	1, r0, r2
            sub!	r1, r2, r1
            jump.ne	@.BB0_9
            jump	@.BB0_3
        .BB0_3:
            add	@CPI0_0[0], r0, r1
            uma.heap_read	r1, r0, r1
            add	@CPI0_1[0], r0, r2
            uma.heap_read	r2, r0, r2
            shl.s	32, r2, r2
            add	r1, r2, r1
            ret
        .BB0_4:
        .tmp2:
            near_call	r0, @__constructor, @.BB0_2
        .tmp3:
            jump	@.BB0_6
        .BB0_5:
        .tmp0:
            near_call	r0, @__selector, @.BB0_2
        .tmp1:
            jump	@.BB0_7
        .BB0_6:
            jump	@.BB0_3
        .BB0_7:
            jump	@.BB0_3
        .BB0_9:
            add	@CPI0_0[0], r0, r1
            uma.heap_read	r1, r0, r1
            add	@CPI0_1[0], r0, r2
            uma.heap_read	r2, r0, r2
            shl.s	32, r2, r2
            add	r1, r2, r1
            add	0, r0, r3
            add	r3, r0, r2
            near_call	r0, @__cxa_throw, @DEFAULT_UNWIND
        .func_end0:

        __constructor:
        .func_begin1:
            nop	stack+=[8]
            add	128, r0, stack-[8]
            add	stack-[8], r0, r2
            add	64, r0, r1
            uma.heap_write	r1, r2, r0
            add	0, r0, r1
            sub!	r1, r1, r1
            jump.ne	@.BB1_4
            jump	@.BB1_5
        .BB1_1:
            add	0, r0, r3
            add	r3, r0, r1
            add	r3, r0, r2
            near_call	r0, @__cxa_throw, @DEFAULT_UNWIND
        .BB1_3:
            nop	stack-=[8]
            ret
        .BB1_4:
            add	0, r0, r2
            add	@CPI1_0[0], r0, r1
            uma.heap_write	r1, r2, r0
            add	@CPI1_2[0], r0, r1
            uma.heap_write	r1, r2, r0
            jump	@.BB1_1
        .BB1_5:
            add	0, r0, stack-[7]
            add	stack-[8], r0, r2
            add	r2, r0, stack-[0]
            add	stack-[7], r0, r1
            add	@CPI1_0[0], r0, r3
            uma.heap_read	r3, r0, r3
            add	r2, r3, r2
            add	r2, r0, stack-[1]
            shr.s	5, r1, r2
            add	r2, r0, stack-[2]
            and	31, r1, r3
            add	r3, r0, stack-[3]
            and	@CPI1_1[0], r1, r1
            add	r1, r0, stack-[4]
            add	0, r0, r1
            sub!	r2, r1, r2
            add	r1, r0, stack-[5]
            jump.eq	@.BB1_9
            jump	@.BB1_6
        .BB1_6:
            add	stack-[5], 0, r1
            add	stack-[2], 0, r2
            add	stack-[0], 0, r3
            add	stack-[1], 0, r4
            shl.s	5, r1, r5
            add	r4, r5, r4
            uma.calldata_read	r4, r0, r4
            add	r3, r5, r3
            uma.heap_write	r3, r4, r0
            add	1, r1, r1
            sub!	r1, r2, r2
            add	r1, r0, stack-[5]
            jump.lt	@.BB1_6
            jump	@.BB1_9
        .BB1_7:
            add	stack-[0], 0, r1
            add	stack-[4], 0, r3
            add	stack-[3], 0, r4
            add	stack-[1], 0, r2
            add	r2, r3, r2
            uma.calldata_read	r2, r0, r2
            shl.s	3, r4, r4
            sub	256, r4, r5
            shr	r2, r5, r2
            shl	r2, r5, r2
            add	r1, r3, r1
            uma.heap_read	r1, r0, r3
            shl	r3, r4, r3
            shr	r3, r4, r3
            or	r2, r3, r2
            uma.heap_write	r1, r2, r0
            jump	@.BB1_8
        .BB1_8:
            add	stack-[8], r0, r3
            add	stack-[7], r0, r2
            add	@CPI1_0[0], r0, r1
            uma.heap_write	r1, r3, r0
            add	@CPI1_2[0], r0, r1
            uma.heap_write	r1, r2, r0
            jump	@.BB1_3
        .BB1_9:
            add	stack-[3], 0, r1
            add	0, r0, r2
            sub!	r1, r2, r1
            jump.ne	@.BB1_7
            jump	@.BB1_8
        .func_end1:

        __selector:
        .func_begin2:
            nop	stack+=[14]
            add	128, r0, stack-[14]
            add	stack-[14], r0, r2
            add	64, r0, r1
            uma.heap_write	r1, r2, r0
            add	@CPI2_0[0], r0, r1
            uma.heap_read	r1, r0, r1
            add	4, r0, r2
            sub!	r1, r2, r1
            add	0, r0, r1
            add.lt	1, r0, r1
            and	1, r1, r1
            add	0, r0, r2
            sub!	r1, r2, r1
            add	0, r0, r1
            add.eq	1, r0, r1
            and	1, r1, r1
            sub!	r1, r2, r1
            jump.ne	@.BB2_4
            jump	@.BB2_5
        .BB2_1:
            add	0, r0, r3
            add	r3, r0, r1
            add	r3, r0, r2
            near_call	r0, @__cxa_throw, @DEFAULT_UNWIND
        .BB2_2:
        .tmp11:
            add	0, r0, r3
            add	r3, r0, r1
            add	r3, r0, r2
            near_call	r0, @__cxa_throw, @DEFAULT_UNWIND
        .BB2_3:
            nop	stack-=[14]
            ret
        .BB2_4:
            add	0, r0, stack-[13]
            jump	@.BB2_7
        .BB2_5:
            add	0, r0, r2
            add	@CPI2_1[0], r0, r1
            uma.heap_write	r1, r2, r0
            add	@CPI2_0[0], r0, r1
            uma.heap_write	r1, r2, r0
            jump	@.BB2_1
        .BB2_6:
            jump	@.BB2_5
        .BB2_7:
            add	stack-[13], r0, r1
            add	@CPI2_1[0], r0, r2
            uma.heap_read	r2, r0, r2
            add	r1, r2, r1
            uma.calldata_read	r1, r0, r1
            add	r1, r0, stack-[5]
            add	0, r0, r1
            sub!	r1, r1, r1
            jump.ne	@.BB2_14
            jump	@.BB2_15
        .BB2_8:
            add	0, r0, r2
            add	1, r0, r1
            sub!	r1, r2, r1
            jump.ne	@.BB2_10
            jump	@.BB2_9
        .BB2_9:
            add	stack-[13], r0, r2
            add	@CPI2_1[0], r0, r1
            uma.heap_write	r1, r2, r0
            add	@CPI2_0[0], r0, r1
            uma.heap_write	r1, r2, r0
            jump	@.BB2_1
        .BB2_10:
            add	@CPI2_0[0], r0, r1
            uma.heap_read	r1, r0, r1
        .tmp9:
            near_call	r0, @abi_decode_bool, @.BB2_2
        .tmp10:
            add	r1, r0, stack-[4]
            jump	@.BB2_11
        .BB2_11:
            add	stack-[4], 0, r1
            add	0, r0, r2
            sub!	r1, r2, r1
            jump.eq	@.BB2_13
            jump	@.BB2_12
        .BB2_12:
            add	stack-[13], r0, r2
            add	@CPI2_1[0], r0, r1
            uma.heap_write	r1, r2, r0
            add	@CPI2_0[0], r0, r1
            uma.heap_write	r1, r2, r0
            jump	@.BB2_1
        .BB2_13:
            add	stack-[14], r0, r1
            add	5, r0, r2
            uma.heap_write	r1, r2, r0
            add	stack-[14], r0, r2
            add	@CPI2_1[0], r0, r1
            uma.heap_write	r1, r2, r0
            add	32, r0, r2
            add	@CPI2_0[0], r0, r1
            uma.heap_write	r1, r2, r0
            jump	@.BB2_3
        .BB2_14:
            add	0, r0, stack-[12]
            jump	@.BB2_16
        .BB2_15:
            add	stack-[5], 0, r1
            shr.s	224, r1, stack-[12]
            jump	@.BB2_16
        .BB2_16:
            add	stack-[12], r0, r2
            add	@CPI2_2[0], r0, r1
            sub!	r1, r2, r1
            jump.eq	@.BB2_8
            jump	@.BB2_17
        .BB2_17:
            add	stack-[13], r0, r1
            add	@CPI2_1[0], r0, r2
            uma.heap_read	r2, r0, r2
            add	r1, r2, r1
            uma.calldata_read	r1, r0, r1
            add	r1, r0, stack-[3]
            add	0, r0, r1
            sub!	r1, r1, r1
            jump.ne	@.BB2_27
            jump	@.BB2_28
        .BB2_18:
            add	0, r0, r2
            add	1, r0, r1
            sub!	r1, r2, r1
            jump.ne	@.BB2_20
            jump	@.BB2_19
        .BB2_19:
            add	stack-[13], r0, r2
            add	@CPI2_1[0], r0, r1
            uma.heap_write	r1, r2, r0
            add	@CPI2_0[0], r0, r1
            uma.heap_write	r1, r2, r0
            jump	@.BB2_1
        .BB2_20:
            add	@CPI2_0[0], r0, r1
            uma.heap_read	r1, r0, r1
        .tmp7:
            near_call	r0, @abi_decode_bool, @.BB2_2
        .tmp8:
            add	r1, r0, stack-[2]
            jump	@.BB2_21
        .BB2_21:
            add	stack-[2], 0, r1
            add	0, r0, r2
            sub!	r1, r2, r1
            jump.eq	@.BB2_23
            jump	@.BB2_22
        .BB2_22:
            add	64, r0, r1
            uma.heap_read	r1, r0, r1
            add	r1, r0, stack-[10]
            add	stack-[10], r0, r1
            add	r1, r0, stack-[1]
            add	0, r0, r1
            sub!	r1, r1, r1
            jump.ne	@.BB2_24
            jump	@.BB2_25
        .BB2_23:
            add	64, r0, r1
            uma.heap_read	r1, r0, r1
            add	r1, r0, stack-[8]
            add	stack-[8], r0, r1
            add	5, r0, r2
            uma.heap_write	r1, r2, r0
            add	stack-[8], r0, r2
            add	@CPI2_1[0], r0, r1
            uma.heap_write	r1, r2, r0
            add	32, r0, r2
            add	@CPI2_0[0], r0, r1
            uma.heap_write	r1, r2, r0
            jump	@.BB2_3
        .BB2_24:
            add	0, r0, stack-[9]
            jump	@.BB2_26
        .BB2_25:
            add	@CPI2_5[0], r0, r1
            add	r1, r0, stack-[9]
            jump	@.BB2_26
        .BB2_26:
            add	stack-[1], 0, r1
            add	stack-[9], r0, r2
            uma.heap_write	r1, r2, r0
            add	stack-[10], r0, r1
            add	4, r1, r1
            add	32, r0, r2
            uma.heap_write	r1, r2, r0
            add	stack-[10], r0, r1
            add	36, r1, r1
            add	5, r0, r2
            uma.heap_write	r1, r2, r0
            add	stack-[10], r0, r1
            add	68, r1, r1
            add	@CPI2_6[0], r0, r2
            uma.heap_write	r1, r2, r0
            add	stack-[10], r0, r2
            add	@CPI2_1[0], r0, r1
            uma.heap_write	r1, r2, r0
            add	100, r0, r2
            add	@CPI2_0[0], r0, r1
            uma.heap_write	r1, r2, r0
            jump	@.BB2_1
        .BB2_27:
            add	0, r0, stack-[11]
            jump	@.BB2_29
        .BB2_28:
            add	stack-[3], 0, r1
            shr.s	224, r1, stack-[11]
            jump	@.BB2_29
        .BB2_29:
            add	stack-[11], r0, r2
            add	@CPI2_3[0], r0, r1
            sub!	r1, r2, r1
            jump.eq	@.BB2_18
            jump	@.BB2_30
        .BB2_30:
            add	stack-[13], r0, r1
            add	@CPI2_1[0], r0, r2
            uma.heap_read	r2, r0, r2
            add	r1, r2, r1
            uma.calldata_read	r1, r0, r1
            add	r1, r0, stack-[0]
            add	0, r0, r1
            sub!	r1, r1, r1
            jump.ne	@.BB2_33
            jump	@.BB2_34
        .BB2_31:
        .tmp5:
            near_call	r0, @external_fun_with_empty_message, @.BB2_2
        .tmp6:
            jump	@.BB2_32
        .BB2_32:
            jump	@.BB2_6
        .BB2_33:
            add	0, r0, stack-[7]
            jump	@.BB2_35
        .BB2_34:
            add	stack-[0], 0, r1
            shr.s	224, r1, stack-[7]
            jump	@.BB2_35
        .BB2_35:
            add	stack-[7], r0, r2
            add	@CPI2_4[0], r0, r1
            sub!	r1, r2, r1
            jump.eq	@.BB2_31
            jump	@.BB2_36
        .BB2_36:
            jump	@.BB2_6
        .func_end2:

        abi_decode_bool:
        .func_begin3:
            nop	stack+=[3]
            add	0, r0, stack-[3]
            add	r1, r0, stack-[2]
            add	stack-[2], r0, r1
            add	@CPI3_0[0], r1, r2
            add	32, r0, r1
            add	@CPI3_1[0], r0, r5
            sub!	r2, r1, r1
            add	0, r0, r1
            add.lt	r5, r0, r1
            and	r2, r5, r4
            add	0, r0, r2
            sub!	r4, r2, r3
            add	0, r0, r3
            add.gt	r5, r0, r3
            sub!	r4, r5, r4
            add	r1, r0, r1
            add.eq	r3, r0, r1
            sub!	r1, r2, r1
            add	0, r0, r1
            add.ne	1, r0, r1
            and	1, r1, r1
            sub!	r1, r2, r1
            jump.ne	@.BB3_4
            jump	@.BB3_5
        .BB3_1:
            add	0, r0, r3
            add	r3, r0, r1
            add	r3, r0, r2
            near_call	r0, @__cxa_throw, @DEFAULT_UNWIND
        .BB3_3:
            add	stack-[3], r0, r1
            nop	stack-=[3]
            ret
        .BB3_4:
            add	0, r0, r2
            add	@CPI3_2[0], r0, r1
            uma.heap_write	r1, r2, r0
            add	@CPI3_3[0], r0, r1
            uma.heap_write	r1, r2, r0
            jump	@.BB3_1
        .BB3_5:
            add	@CPI3_2[0], r0, r1
            uma.heap_read	r1, r0, r1
            add	4, r1, r1
            uma.calldata_read	r1, r0, r1
            add	r1, r0, stack-[1]
            add	stack-[1], r0, r1
            add	0, r0, r2
            sub!	r1, r2, r3
            add	0, r0, r3
            add.eq	1, r0, r3
            and	1, r3, r3
            sub!	r3, r2, r3
            add	0, r0, r3
            add.eq	1, r0, r3
            and	1, r3, r3
            sub!	r1, r3, r1
            add	0, r0, r1
            add.eq	1, r0, r1
            and	1, r1, r1
            sub!	r1, r2, r1
            add	0, r0, r1
            add.eq	1, r0, r1
            and	1, r1, r1
            sub!	r1, r2, r1
            jump.eq	@.BB3_7
            jump	@.BB3_6
        .BB3_6:
            add	0, r0, r2
            add	@CPI3_2[0], r0, r1
            uma.heap_write	r1, r2, r0
            add	@CPI3_3[0], r0, r1
            uma.heap_write	r1, r2, r0
            jump	@.BB3_1
        .BB3_7:
            add	stack-[1], r0, r1
            add	r1, r0, stack-[3]
            jump	@.BB3_3
        .func_end3:

        external_fun_with_empty_message:
        .func_begin4:
            nop	stack+=[5]
            add	0, r0, r1
            sub!	r1, r1, r1
            jump.ne	@.BB4_4
            jump	@.BB4_5
        .BB4_1:
            add	0, r0, r3
            add	r3, r0, r1
            add	r3, r0, r2
            near_call	r0, @__cxa_throw, @DEFAULT_UNWIND
        .BB4_2:
        .tmp14:
            add	0, r0, r3
            add	r3, r0, r1
            add	r3, r0, r2
            near_call	r0, @__cxa_throw, @DEFAULT_UNWIND
        .BB4_4:
            add	0, r0, r2
            add	@CPI4_1[0], r0, r1
            uma.heap_write	r1, r2, r0
            add	@CPI4_0[0], r0, r1
            uma.heap_write	r1, r2, r0
            jump	@.BB4_1
        .BB4_5:
            add	@CPI4_0[0], r0, r1
            uma.heap_read	r1, r0, r1
        .tmp12:
            near_call	r0, @abi_decode_bool, @.BB4_2
        .tmp13:
            add	r1, r0, stack-[1]
            jump	@.BB4_6
        .BB4_6:
            add	stack-[1], 0, r1
            add	0, r0, r2
            sub!	r1, r2, r1
            jump.eq	@.BB4_8
            jump	@.BB4_7
        .BB4_7:
            add	64, r0, r1
            uma.heap_read	r1, r0, r1
            add	r1, r0, stack-[5]
            add	stack-[5], r0, r1
            add	r1, r0, stack-[0]
            add	0, r0, r1
            sub!	r1, r1, r1
            jump.ne	@.BB4_9
            jump	@.BB4_10
        .BB4_8:
            add	64, r0, r1
            uma.heap_read	r1, r0, r1
            add	r1, r0, stack-[3]
            add	stack-[3], r0, r1
            add	5, r0, r2
            uma.heap_write	r1, r2, r0
            add	stack-[3], r0, r2
            add	@CPI4_1[0], r0, r1
            uma.heap_write	r1, r2, r0
            add	32, r0, r2
            add	@CPI4_0[0], r0, r1
            uma.heap_write	r1, r2, r0
            add	1, r0, r2
            add	@CPI4_2[0], r0, r1
            uma.heap_write	r1, r2, r0
            jump	@.BB4_1
        .BB4_9:
            add	0, r0, stack-[4]
            jump	@.BB4_11
        .BB4_10:
            add	@CPI4_3[0], r0, r1
            add	r1, r0, stack-[4]
            jump	@.BB4_11
        .BB4_11:
            add	stack-[0], 0, r1
            add	stack-[4], r0, r2
            uma.heap_write	r1, r2, r0
            add	stack-[5], r0, r1
            add	4, r1, r1
            add	32, r0, r2
            uma.heap_write	r1, r2, r0
            add	stack-[5], r0, r1
            add	36, r1, r1
            add	0, r0, r2
            uma.heap_write	r1, r2, r0
            add	stack-[5], r0, r2
            add	@CPI4_1[0], r0, r1
            uma.heap_write	r1, r2, r0
            add	68, r0, r2
            add	@CPI4_0[0], r0, r1
            uma.heap_write	r1, r2, r0
            jump	@.BB4_1
        .func_end4:

        __cxa_throw:
            revert

            .note.GNU-stack
            .rodata
        CPI0_0:
            .cell 16777184
        CPI0_1:
            .cell 16777152
        CPI0_2:
            .cell 16777120
        CPI1_0:
            .cell 16777184
        CPI1_1:
            .cell -32
        CPI1_2:
            .cell 16777152
        CPI2_0:
            .cell 16777152
        CPI2_1:
            .cell 16777184
        CPI2_2:
            .cell 1559963207
        CPI2_3:
            .cell 2109228076
        CPI2_4:
            .cell 3138363696
        CPI2_5:
            .cell 3963877391197344453575983046348115674221700746820753546331534351508065746944
        CPI2_6:
            .cell 31411796921273332039540167021977234770400022203154653039219347659833896599552
        CPI3_0:
            .cell -4
        CPI3_1:
            .cell -57896044618658097711785492504343953926634992332820282019728792003956564819968
        CPI3_2:
            .cell 16777184
        CPI3_3:
            .cell 16777152
        CPI4_0:
            .cell 16777152
        CPI4_1:
            .cell 16777184
        CPI4_2:
            .cell 16777120
        CPI4_3:
            .cell 3963877391197344453575983046348115674221700746820753546331534351508065746944
    "#;

    #[test]
    fn run_returndata_on_revert() {
        set_tracing_mode(VmTracingOptions::ManualVerbose);
        run_inner(
            &hex::decode("bb0fa1300000000000000000000000000000000000000000000000000000000000000001")
                .unwrap(),
            VmLaunchOption::Default,
            ENSURE_PROPER_RETURN_ON_REVERT,
        );
    }

    const KECCAK256_SYSTEM_ASM: &'static str = r#"
	.text
	.file	"Test_270"
	.globl	__entry
__entry:
.func_begin0:
	add	@CPI0_0[0], r0, r4
	uma.heap_write	r4, r1, r0
	add	@CPI0_1[0], r0, r1
	uma.heap_write	r1, r2, r0
	and	1, r3, r1
	add	1, r0, r2
	sub!	r1, r2, r1
	jump.ne	@.BB0_2
	add	128, r0, r1
	add	64, r0, r2
	uma.heap_write	r2, r1, r0
	ret.ok.to_label	r1, @DEFAULT_FAR_RETURN
.BB0_2:
	near_call	r0, @__selector, @DEFAULT_UNWIND
.func_end0:

__selector:
.func_begin1:
	add	128, r0, r1
	add	64, r0, r2
	uma.heap_write	r2, r1, r0
	add	@CPI1_0[0], r0, r1
	context.code_source	r2
	and	r2, r1, r1
	context.this	r2
	sub!	r1, r2, r1
	jump.eq	@.BB1_2
	add	0, r0, r1
	ret.revert.to_label	r1, @DEFAULT_FAR_REVERT
.BB1_2:
	add	@CPI1_1[0], r0, r1
	uma.heap_read	r1, r0, r6
	add	1024, r0, r1
	sub!	r6, r1, r1
	jump.le	@.BB1_4
	add	0, r0, r1
	ret.revert.to_label	r1, @DEFAULT_FAR_REVERT
.BB1_4:
	and	65535, r6, r1
	div.s	136, r1, r1, r2
	sub	136, r2, r1
	and	65535, r1, r3
	add	r6, r3, r1
	and	65535, r1, r2
	div.s	136, r2, r5, r2
	mul	136, r5, r2, r4
	sub	r1, r2, r2
	and	65535, r2, r2
	add	0, r0, r4
	sub!	r2, r4, r2
	jump.eq	@.BB1_6
	add	@CPI1_2[0], r0, r1
	uma.heap_write	r4, r1, r0
	add	1, r0, r1
	add	4, r0, r2
	uma.heap_write	r2, r1, r0
	add	@CPI1_3[0], r0, r1
	ret.revert.to_label	r1, @DEFAULT_FAR_REVERT
.BB1_6:
	and	31, r1, r2
	sub!	r2, r4, r2
	add	0, r0, r2
	add.ne	1, r0, r2
	add	@CPI1_4[0], r0, r8
	and	31, r6, r7
	add	@CPI1_5[0], r0, r9
	uma.heap_read	r9, r0, r9
	shr.s	5, r6, r10
	sub!	r10, r4, r11
	jump.eq	@.BB1_16
.BB1_7:
	shl.s	5, r4, r11
	add	r9, r11, r12
	uma.calldata_read	r12, r0, r12
	add	128, r11, r11
	uma.heap_write	r11, r12, r0
	add	1, r4, r4
	sub!	r4, r10, r11
	jump.lt	@.BB1_7
	jump	@.BB1_16
.BB1_8:
	and	r6, r8, r6
	add	r9, r6, r8
	shl.s	3, r7, r7
	add	128, r6, r6
	uma.heap_read	r6, r0, r9
	shl	r9, r7, r9
	shr	r9, r7, r9
	uma.calldata_read	r8, r0, r8
	sub	256, r7, r7
	shr	r8, r7, r8
	shl	r8, r7, r7
	or	r7, r9, r7
	uma.heap_write	r6, r7, r0
.BB1_9:
	and	65535, r5, r5
	add	@CPI1_6[0], r0, r7
	add	@CPI1_1[0], r0, r6
	uma.heap_read	r6, r0, r6
	add	128, r6, r8
	add	1, r0, r6
	sub!	r3, r6, r3
	jump.ne	@.BB1_15
.BB1_10:
	uma.heap_write	r8, r7, r0
	mul	100, r5, r3, r7
	add	100, r3, r3
    context.ergs_left r6
	sub!	r6, r3, r6
	jump.ge	@.BB1_12
	add	0, r0, r1
	ret.revert.to_label	r1, @DEFAULT_FAR_REVERT
.BB1_12:
	and	1, r2, r2
	shr.s	5, r1, r1
	add	r1, r2, r1
	shl.s	32, r1, r1
	add	@CPI1_9[0], r0, r2
	and	r1, r2, r1
	shl.s	192, r5, r2
	or	r1, r2, r1
	add	@CPI1_10[0], r0, r2
	or	r1, r2, r1
	precompile	r1, r3, r1
	sub!	r1, r4, r1
	jump.ne	@.BB1_14
	add	0, r0, r1
	ret.revert.to_label	r1, @DEFAULT_FAR_REVERT
.BB1_14:
	add	@CPI1_11[0], r0, r1
	ret.ok.to_label	r1, @DEFAULT_FAR_RETURN
.BB1_15:
	add	@CPI1_7[0], r0, r3
	uma.heap_write	r8, r3, r0
	add	@CPI1_8[0], r0, r7
	add	127, r1, r8
	jump	@.BB1_10
.BB1_16:
	add	0, r0, r4
	sub!	r7, r4, r10
	jump.ne	@.BB1_8
	jump	@.BB1_9
.func_end1:

	.note.GNU-stack
	.rodata
CPI0_0:
	.cell 16777184
CPI0_1:
	.cell 16777152
CPI1_0:
	.cell 1461501637330902918203684832716283019655932542975
CPI1_1:
	.cell 16777152
CPI1_2:
	.cell 35408467139433450592217433187231851964531694900788300625387963629091585785856
CPI1_3:
	.cell 154618822656
CPI1_4:
	.cell -32
CPI1_5:
	.cell 16777184
CPI1_6:
	.cell -57443731770074831323412168344153766786583156455220123566449660816425654157312
CPI1_7:
	.cell 452312848583266388373324160190187140051835877600158453279131187530910662656
CPI1_8:
	.cell -57896044618658097711785492504343953926634992332820282019728792003956564819968
CPI1_9:
	.cell 18446744069414584320
CPI1_10:
	.cell 79228162514264337593543950340
CPI1_11:
	.cell 137438953472
    "#;

    #[test]
    fn run_keccak_system_contract() {
        set_tracing_mode(VmTracingOptions::ManualVerbose);
        let mut ctx = VmExecutionContext::default();
        ctx.msg_sender = Address::from_low_u64_be(0x1_000_000);
        ctx.this_address = Address::from_low_u64_be(0x10);
        dbg!(ctx.msg_sender);
        dbg!(ctx.this_address);
        run_inner_with_context(
            &hex::decode("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap(),
            // hex::decode("00ff").unwrap(), 
            VmLaunchOption::Default,
            KECCAK256_SYSTEM_ASM,
            ctx
        );
    }

    const SHA256_SYSTEM_ASM: &'static str = r#"
    
	.text
	.file	"Test_293"
	.globl	__entry
__entry:
.func_begin0:
	add	@CPI0_0[0], r0, r4
	uma.heap_write	r4, r1, r0
	add	@CPI0_1[0], r0, r1
	uma.heap_write	r1, r2, r0
	and	1, r3, r1
	add	1, r0, r2
	sub!	r1, r2, r1
	jump.ne	@.BB0_2
	add	128, r0, r1
	add	64, r0, r2
	uma.heap_write	r2, r1, r0
	ret.ok.to_label	r1, @DEFAULT_FAR_RETURN
.BB0_2:
	near_call	r0, @__selector, @DEFAULT_UNWIND
.func_end0:

__selector:
.func_begin1:
	add	128, r0, r1
	add	64, r0, r2
	uma.heap_write	r2, r1, r0
	add	@CPI1_0[0], r0, r1
	context.code_source	r2
	and	r2, r1, r1
	context.this	r2
	sub!	r1, r2, r1
	jump.eq	@.BB1_2
	add	0, r0, r1
	ret.revert.to_label	r1, @DEFAULT_FAR_REVERT
.BB1_2:
	add	@CPI1_1[0], r0, r1
	uma.heap_read	r1, r0, r2
	add	1024, r0, r1
	sub!	r2, r1, r1
	jump.le	@.BB1_4
	add	0, r0, r1
	ret.revert.to_label	r1, @DEFAULT_FAR_REVERT
.BB1_4:
	add	@CPI1_2[0], r0, r1
	add	0, r0, r3
	sub!	r3, r3, r4
	jump.eq	@.BB1_6
	add	@CPI1_3[0], r0, r1
	uma.heap_write	r3, r1, r0
	add	1, r0, r1
	add	4, r0, r2
	uma.heap_write	r2, r1, r0
	add	@CPI1_4[0], r0, r1
	ret.revert.to_label	r1, @DEFAULT_FAR_REVERT
.BB1_6:
	and	r2, r1, r1
	and	63, r2, r4
	add	64, r1, r5
	add	64, r5, r6
	add	55, r0, r1
	sub!	r4, r1, r1
	add	r5, r0, r1
	add.gt	r6, r0, r1
	add	0, r0, r4
	add.gt	1, r0, r4
	and	1, r4, r4
	shr.s	6, r5, r8
	add	@CPI1_5[0], r0, r6
	and	31, r2, r5
	add	@CPI1_6[0], r0, r7
	uma.heap_read	r7, r0, r7
	shr.s	5, r2, r9
	sub!	r9, r3, r10
	jump.eq	@.BB1_14
.BB1_7:
	shl.s	5, r3, r10
	add	r7, r10, r11
	uma.calldata_read	r11, r0, r11
	add	128, r10, r10
	uma.heap_write	r10, r11, r0
	add	1, r3, r3
	sub!	r3, r9, r10
	jump.lt	@.BB1_7
	jump	@.BB1_14
.BB1_8:
	and	r2, r6, r2
	add	r7, r2, r6
	shl.s	3, r5, r5
	add	128, r2, r2
	uma.heap_read	r2, r0, r7
	shl	r7, r5, r7
	shr	r7, r5, r7
	uma.calldata_read	r6, r0, r6
	sub	256, r5, r5
	shr	r6, r5, r6
	shl	r6, r5, r5
	or	r5, r7, r5
	uma.heap_write	r2, r5, r0
.BB1_9:
	add	@CPI1_1[0], r0, r2
	uma.heap_read	r2, r0, r5
	add	128, r5, r5
	add	@CPI1_7[0], r0, r6
	uma.heap_write	r5, r6, r0
	add	120, r1, r5
	uma.heap_read	r2, r0, r2
	shl.s	195, r2, r2
	uma.heap_write	r5, r2, r0
	mul	50, r4, r2, r5
	add	100, r2, r2
	add	@CPI1_8[0], r0, r5
	and	r2, r5, r2
	context.ergs_left	r5
	sub!	r5, r2, r5
	jump.ge	@.BB1_11
	add	0, r0, r1
	ret.revert.to_label	r1, @DEFAULT_FAR_REVERT
.BB1_11:
	shl.s	27, r1, r1
	add	@CPI1_9[0], r0, r5
	and	r1, r5, r1
	shl.s	192, r4, r4
	or	r1, r4, r1
	add	@CPI1_10[0], r0, r4
	or	r1, r4, r1
	precompile	r1, r2, r1
	sub!	r1, r3, r1
	jump.ne	@.BB1_13
	add	0, r0, r1
	ret.revert.to_label	r1, @DEFAULT_FAR_REVERT
.BB1_13:
	add	@CPI1_11[0], r0, r1
	ret.ok.to_label	r1, @DEFAULT_FAR_RETURN
.BB1_14:
	add	r8, r4, r4
	add	0, r0, r3
	sub!	r5, r3, r8
	jump.ne	@.BB1_8
	jump	@.BB1_9
.func_end1:

	.note.GNU-stack
	.rodata
CPI0_0:
	.cell 16777184
CPI0_1:
	.cell 16777152
CPI1_0:
	.cell 1461501637330902918203684832716283019655932542975
CPI1_1:
	.cell 16777152
CPI1_2:
	.cell -64
CPI1_3:
	.cell 35408467139433450592217433187231851964531694900788300625387963629091585785856
CPI1_4:
	.cell 154618822656
CPI1_5:
	.cell -32
CPI1_6:
	.cell 16777184
CPI1_7:
	.cell -57896044618658097711785492504343953926634992332820282019728792003956564819968
CPI1_8:
	.cell 4294967294
CPI1_9:
	.cell 18446744069414584320
CPI1_10:
	.cell 79228162514264337593543950340
CPI1_11:
	.cell 137438953472
    "#;

    #[test]
    fn run_sha256_system_contract() {
        set_tracing_mode(VmTracingOptions::ManualVerbose);
        let mut ctx = VmExecutionContext::default();
        ctx.msg_sender = Address::from_low_u64_be(0x1_000_000);
        ctx.this_address = Address::from_low_u64_be(0x11);
        dbg!(ctx.msg_sender);
        dbg!(ctx.this_address);
        run_inner_with_context(
            &hex::decode("00").unwrap(),
            // hex::decode("00ff").unwrap(),
            VmLaunchOption::Default,
            SHA256_SYSTEM_ASM,
            ctx,
        );
    }

    const ECRECOVER_SYSTEM_ASM: &'static str = r#"

	.text
	.file	"Test_268"
	.globl	__entry
__entry:
.func_begin0:
	add	@CPI0_0[0], r0, r4
	uma.heap_write	r4, r1, r0
	add	@CPI0_1[0], r0, r1
	uma.heap_write	r1, r2, r0
	and	1, r3, r1
	add	1, r0, r2
	sub!	r1, r2, r1
	jump.ne	@.BB0_2
	add	128, r0, r1
	add	64, r0, r2
	uma.heap_write	r2, r1, r0
	ret.ok.to_label	r1, @DEFAULT_FAR_RETURN
.BB0_2:
	near_call	r0, @__selector, @DEFAULT_UNWIND
.func_end0:

__selector:
.func_begin1:
	add	128, r0, r1
	add	64, r0, r2
	uma.heap_write	r2, r1, r0
	add	@CPI1_0[0], r0, r2
	context.code_source	r3
	and	r3, r2, r2
	context.this	r3
	sub!	r2, r3, r2
	jump.eq	@.BB1_2
	add	0, r0, r1
	ret.revert.to_label	r1, @DEFAULT_FAR_REVERT
.BB1_2:
	add	@CPI1_1[0], r0, r2
	uma.heap_read	r2, r0, r2
	sub!	r2, r1, r2
	jump.eq	@.BB1_4
	add	0, r0, r1
	ret.revert.to_label	r1, @DEFAULT_FAR_REVERT
.BB1_4:
	add	@CPI1_2[0], r0, r2
	uma.heap_read	r2, r0, r5
	add	96, r5, r2
	uma.calldata_read	r2, r0, r3
	add	64, r5, r2
	uma.calldata_read	r2, r0, r4
	add	32, r5, r2
	uma.calldata_read	r2, r0, r7
	add	1, r0, r2
	sub!	r7, r2, r6
	jump.le	@.BB1_7
	add	@CPI1_3[0], r0, r6
	add	r7, r6, r7
	sub!	r7, r2, r6
	jump.le	@.BB1_7
	add	0, r0, r1
	ret.revert.to_label	r1, @DEFAULT_FAR_REVERT
.BB1_7:
	add	0, r0, r6
	sub!	r3, r6, r8
	jump.ne	@.BB1_9
	add	0, r0, r1
	ret.revert.to_label	r1, @DEFAULT_FAR_REVERT
.BB1_9:
	sub!	r4, r6, r8
	jump.ne	@.BB1_11
	add	0, r0, r1
	ret.revert.to_label	r1, @DEFAULT_FAR_REVERT
.BB1_11:
	add	@CPI1_4[0], r0, r8
	sub!	r3, r8, r8
	jump.le	@.BB1_13
	add	0, r0, r1
	ret.revert.to_label	r1, @DEFAULT_FAR_REVERT
.BB1_13:
	add	@CPI1_5[0], r0, r8
	sub!	r4, r8, r8
	jump.le	@.BB1_15
	add	0, r0, r1
	ret.revert.to_label	r1, @DEFAULT_FAR_REVERT
.BB1_15:
	uma.calldata_read	r5, r0, r5
	uma.heap_write	r1, r5, r0
	add	160, r0, r1
	uma.heap_write	r1, r7, r0
	add	192, r0, r1
	uma.heap_write	r1, r4, r0
	add	224, r0, r1
	uma.heap_write	r1, r3, r0
	add	3000, r0, r1
	context.ergs_left	r3
	sub!	r3, r1, r3
	jump.ge	@.BB1_17
	add	0, r0, r1
	ret.revert.to_label	r1, @DEFAULT_FAR_REVERT
.BB1_17:
	add	@CPI1_6[0], r0, r3
	precompile	r3, r1, r1
	sub!	r1, r6, r1
	jump.ne	@.BB1_19
	add	0, r0, r1
	ret.revert.to_label	r1, @DEFAULT_FAR_REVERT
.BB1_19:
	uma.heap_read	r6, r0, r1
	sub!	r1, r2, r1
	jump.eq	@.BB1_21
	add	0, r0, r1
	ret.revert.to_label	r1, @DEFAULT_FAR_REVERT
.BB1_21:
	add	@CPI1_7[0], r0, r1
	ret.ok.to_label	r1, @DEFAULT_FAR_RETURN
.func_end1:

	.note.GNU-stack
	.rodata
CPI0_0:
	.cell 16777184
CPI0_1:
	.cell 16777152
CPI1_0:
	.cell 1461501637330902918203684832716283019655932542975
CPI1_1:
	.cell 16777152
CPI1_2:
	.cell 16777184
CPI1_3:
	.cell -27
CPI1_4:
	.cell 57896044618658097711785492504343953926418782139537452191302581570759080747168
CPI1_5:
	.cell -432420386565659656852420866394968145600
CPI1_6:
	.cell 79228162514264337610723819524
CPI1_7:
	.cell 137438953504
    "#;

    #[test]
    fn test_run_ecrecover_system_contract() {
        set_tracing_mode(VmTracingOptions::ManualVerbose);
        run_ecrecover_system_contract()
    }

    pub fn run_ecrecover_system_contract() {
        set_tracing_mode(VmTracingOptions::None);
        let mut ctx = VmExecutionContext::default();
        ctx.msg_sender = Address::from_low_u64_be(0x1_000_000);
        ctx.this_address = Address::from_low_u64_be(0x12);
        dbg!(ctx.msg_sender);
        dbg!(ctx.this_address);
        let hash = hex::decode("1da44b586eb0729ff70a73c326926f6ed5a25f5b056e7f47fbc6e58d86871655")
            .unwrap();
        let recovery_byte = 0x1c;
        let r = hex::decode("b91467e570a6466aa9e9876cbcd013baba02900b8979d43fe208a4a4f339f5fd")
            .unwrap();
        let s = hex::decode("6007e74cd82e037b800186422fc2da167c747ef045e5d18a5f5d4300f8e1a029")
            .unwrap();
        let mut calldata = hash;
        calldata.extend(std::iter::repeat(0x00).take(31));
        calldata.push(recovery_byte);
        calldata.extend(r);
        calldata.extend(s);

        run_inner_with_context(
            &calldata,
            VmLaunchOption::Default,
            ECRECOVER_SYSTEM_ASM,
            ctx,
        );
    }
}
