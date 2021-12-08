use super::*;

use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::slice::SliceIndex;

use serde::{Deserialize, Serialize};
use zk_evm::opcodes::REGISTERS_COUNT;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ContractSourceDebugInfo {
    pub assembly_code: String,
    pub pc_line_mapping: HashMap<usize, usize>,
    pub active_lines: HashSet<usize>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VmExecutionStep {
    contract_address: String,
    registers: [String; REGISTERS_COUNT],
    pc: u16,
    sp: u16,
    set_flags: Vec<String>,
    skip_cycle: bool,
    code_page_index: u32,
    heap_page_index: u32,
    stack_page_index: u32,
    calldata_page_index: u32,
    returndata_page_index: u32,
    register_interactions: HashMap<usize, MemoryAccessType>,
    memory_interactions: Vec<MemoryInteraction>,
    error: Option<String>
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum MemoryType {
    heap,
    stack,
    calldata,
    returndata,
    code,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum MemoryAccessType {
    Read,
    Write
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MemoryInteraction {
    memory_type: MemoryType,
    page: u32,
    address: u16,
    value: String,
    direction: MemoryAccessType
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VmTrace {
    steps: Vec<VmExecutionStep>,
    sources: HashMap<String, ContractSourceDebugInfo>,
}

use zk_evm::testing::*;
use crate::default_environment::*;

pub fn run_text_assembly_full_trace(assembly: String, calldata: Vec<[u8; 32]>, num_cycles: usize) -> VmTrace {
    let vm_assembly = Assembly::try_from(assembly.clone()).expect("must get a valid assembly as the input");

    let debug_info = ContractSourceDebugInfo {
        assembly_code: vm_assembly.assembly_code.clone(),
        pc_line_mapping: vm_assembly.pc_line_mapping.clone(),
        active_lines: HashSet::new()
    };

    let assembly = vm_assembly.compile_to_bytecode(); 

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


    let mut tracer = VmDebugTracer::new(debug_info);

    for _ in 0..num_cycles {
        vm.cycle(&mut tracer);
    }

    let VmDebugTracer {steps, debug_info, ..} = tracer;

    let mut sources = HashMap::new();
    sources.insert(DEFAULT_CALLEE_HEX.to_owned(), debug_info);

    let full_trace = VmTrace {
        steps,
        sources,
    };

    full_trace
}

fn error_flags_into_description(flags: &ErrorFlags) -> Vec<String> {
    if flags.is_empty() {
        return vec![]
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

#[test]
fn run_something() {
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

    let trace = run_text_assembly_full_trace(
        crate::tests::superior_tests::TIC_TAC_TOE_ASM.to_owned(),
        vec![input], 
        10
    );

    let _ = std::fs::remove_file("tmp.json");
    let mut file = std::fs::File::create("tmp.json").unwrap();
    let json = serde_json::to_string(&trace).unwrap();

    file.write_all(json.as_bytes()).unwrap();
}



pub struct VmDebugTracer<
    'a,
    S: zk_evm::abstractions::Storage, 
    M: zk_evm::abstractions::Memory, 
    EV: zk_evm::abstractions::EventSink,
    PP: zk_evm::abstractions::PrecompilesProcessor,
    DP: zk_evm::abstractions::DecommittmentProcessor,
    WT: zk_evm::witness_trace::VmWitnessTracer,
> {
    pub debug_info: ContractSourceDebugInfo,
    regs_before: Option<[U256; REGISTERS_COUNT]>,
    aux_info: Option<AuxTracingInformation>,
    callstack_info: Option<CallStackEntry>,
    cycle_number: u32,
    pub steps: Vec<VmExecutionStep>,
    _marker: std::marker::PhantomData<VmState<'a, S, M, EV, PP, DP, WT>>
}

impl<
    'a, 
    S: zk_evm::abstractions::Storage, 
    M: zk_evm::abstractions::Memory, 
    EV: zk_evm::abstractions::EventSink,
    PP: zk_evm::abstractions::PrecompilesProcessor,
    DP: zk_evm::abstractions::DecommittmentProcessor,
    WT: zk_evm::witness_trace::VmWitnessTracer,
> std::fmt::Debug for VmDebugTracer<'a, S, M, EV, PP, DP, WT> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VmDebugTracer")
            .finish()
    }
}

impl<
    'a, 
    S: zk_evm::abstractions::Storage, 
    M: zk_evm::abstractions::Memory, 
    EV: zk_evm::abstractions::EventSink,
    PP: zk_evm::abstractions::PrecompilesProcessor,
    DP: zk_evm::abstractions::DecommittmentProcessor,
    WT: zk_evm::witness_trace::VmWitnessTracer,
> VmDebugTracer<'a, S, M, EV, PP, DP, WT> {
    pub fn new(debug_info: ContractSourceDebugInfo) -> Self {
        Self {
            debug_info,
            regs_before: None,
            aux_info: None,
            callstack_info: None,
            cycle_number: 0u32,
            steps: vec![],
            _marker: std::marker::PhantomData
        }
    }
}

impl<
    'a,
    S: zk_evm::abstractions::Storage,  
    EV: zk_evm::abstractions::EventSink,
    PP: zk_evm::abstractions::PrecompilesProcessor,
    DP: zk_evm::abstractions::DecommittmentProcessor,
    WT: zk_evm::witness_trace::VmWitnessTracer,
> zk_evm::abstractions::DebugTracer<VmState<'a, S, SimpleMemory, EV, PP, DP, WT>, AuxTracingInformation, ()> for 
    VmDebugTracer<'a, S, SimpleMemory, EV, PP, DP, WT> 
{
    fn perform_before_execution(&mut self, main: &VmState<'a, S, SimpleMemory, EV, PP, DP, WT>, aux: AuxTracingInformation) {
        debug_assert!(self.aux_info.is_none());
        debug_assert!(self.regs_before.is_none());

        // we need to know
        // - register reads
        // - memory reads

        let skip_cycle = aux.skip_cycle;
        let errors = error_flags_into_description(&aux.error_flags_collection);
        let current_context = main.callstack.get_current_stack();
        let current_pc = current_context.pc.0;
        let current_sp = current_context.sp.0;
        let contract_address = format!("0x{:x}", current_context.contract_address);
        let code_page = current_context.code_page.0;
        let base_memory_page = current_context.base_memory_page.0;
        let calldata_page = current_context.calldata_page.0;
        let calldata_offset = current_context.calldata_offset.0;
        let calldata_len = current_context.calldata_len.0;
        let returndata_page = current_context.returndata_page.0;
        let returndata_offset = current_context.returndata_offset.0;
        let returndata_len = current_context.returndata_len.0;
        self.callstack_info = Some(current_context.clone());
        drop(current_context);
        self.debug_info.active_lines.insert(current_pc as usize);
        let flags = flags_into_description(&main.flags);

        self.regs_before = Some(main.registers);
        
        let registers = main.registers.map(|el| format!("0x{:x}", el));

        let error = if let Some(e) = errors.first() {
            Some(e.clone())
        } else {
            None
        };
        let mut trace_step = VmExecutionStep {
            contract_address,
            registers,
            pc: current_pc,
            sp: current_sp,
            set_flags: flags,
            skip_cycle,
            code_page_index: code_page,
            heap_page_index: CallStackEntry::heap_page_from_base(MemoryPage(base_memory_page)).0,
            stack_page_index: CallStackEntry::stack_page_from_base(MemoryPage(base_memory_page)).0,
            calldata_page_index: calldata_page,
            returndata_page_index: returndata_page,
            register_interactions: HashMap::new(),
            memory_interactions: vec![],
            error
        };
        
        if let Some(mem) = aux.src0_memory_location {
            let MemoryLocation { page, index } = mem;
            let page = page.0;
            let index = index.0;
            let mem_interaction = match page {
                page if page == CallStackEntry::heap_page_from_base(MemoryPage(base_memory_page)).0 ||
                page == CallStackEntry::stack_page_from_base(MemoryPage(base_memory_page)).0 ||
                page == code_page => {
                    let memory_type = if page == CallStackEntry::heap_page_from_base(MemoryPage(base_memory_page)).0 {
                        MemoryType::heap
                    } else if page == CallStackEntry::stack_page_from_base(MemoryPage(base_memory_page)).0 {
                        MemoryType::stack
                    } else if page == code_page {
                        MemoryType::code
                    } else {
                        unreachable!()
                    };

                    let value = main.memory.inner.get(&page).unwrap_or(&vec![]).get(index as usize).copied().unwrap_or(U256::zero());
                    let value = format!("0x{:x}", value);
                    let mem_interaction = MemoryInteraction {
                        memory_type,
                        page,
                        address: index,
                        value,
                        direction: MemoryAccessType::Read
                    };

                    mem_interaction
                },
                page if page == calldata_page ||
                page == returndata_page => {
                    let (memory_type, offset, len) = if page == calldata_page {
                        (MemoryType::calldata, calldata_offset, calldata_len)
                    } else if page == returndata_page {
                        (MemoryType::returndata, returndata_offset, returndata_len)
                    } else {
                        unreachable!()
                    };

                    let value = main.memory.inner.get(&page).unwrap_or(&vec![]).get(index as usize).copied().unwrap_or(U256::zero());
                    let value = format!("0x{:x}", value);

                    let (index, of) = index.overflowing_sub(offset);
                    assert!(!of);
                    assert!(index < len);
                    let mem_interaction = MemoryInteraction {
                        memory_type,
                        page,
                        address: index,
                        value,
                        direction: MemoryAccessType::Read
                    };

                    mem_interaction
                }
                _ => {
                    unreachable!()
                }
            };

            trace_step.memory_interactions.push(mem_interaction);
        } else {
            if aux.src_0_reg != 0 {
                trace_step.register_interactions.insert(aux.src_0_reg as usize, MemoryAccessType::Read);
            }
        }
        if aux.src_1_reg != 0 {
            trace_step.register_interactions.insert(aux.src_1_reg as usize, MemoryAccessType::Read);
        }

        self.aux_info = Some(aux);
        self.steps.push(trace_step);
    }

    fn perform_after_execution(&mut self, main: &VmState<'a, S, SimpleMemory, EV, PP, DP, WT>, _aux: ()) {
        let aux = self.aux_info.take().unwrap();
        let regs_before = self.regs_before.take().unwrap();
        let current_context = self.callstack_info.take().unwrap();

        let code_page = current_context.code_page.0;
        let base_memory_page = current_context.base_memory_page.0;
        let calldata_page = current_context.calldata_page.0;
        let calldata_offset = current_context.calldata_offset.0;
        let calldata_len = current_context.calldata_len.0;
        let returndata_page = current_context.returndata_page.0;
        let returndata_offset = current_context.returndata_offset.0;
        let returndata_len = current_context.returndata_len.0;

        // - register writes
        // - memory writes

        let trace_step = self.steps.last_mut().unwrap();

        if let Some(mem) = aux.dst0_memory_location {
            let MemoryLocation { page, index } = mem;
            let page = page.0;
            let index = index.0;
            let mem_interaction = match page {
                page if page == CallStackEntry::heap_page_from_base(MemoryPage(base_memory_page)).0 ||
                page == CallStackEntry::stack_page_from_base(MemoryPage(base_memory_page)).0 ||
                page == code_page => {
                    let memory_type = if page == CallStackEntry::heap_page_from_base(MemoryPage(base_memory_page)).0 {
                        MemoryType::heap
                    } else if page == CallStackEntry::stack_page_from_base(MemoryPage(base_memory_page)).0 {
                        MemoryType::stack
                    } else if page == code_page {
                        MemoryType::code
                    } else {
                        unreachable!()
                    };

                    let value = main.memory.inner.get(&page).unwrap_or(&vec![]).get(index as usize).copied().unwrap_or(U256::zero());
                    let value = format!("0x{:x}", value);
                    let mem_interaction = MemoryInteraction {
                        memory_type,
                        page,
                        address: index,
                        value,
                        direction: MemoryAccessType::Write
                    };

                    mem_interaction
                },
                page if page == calldata_page ||
                page == returndata_page => {
                    let (memory_type, offset, len) = if page == calldata_page {
                        (MemoryType::calldata, calldata_offset, calldata_len)
                    } else if page == returndata_page {
                        (MemoryType::returndata, returndata_offset, returndata_len)
                    } else {
                        unreachable!()
                    };

                    let value = main.memory.inner.get(&page).unwrap_or(&vec![]).get(index as usize).copied().unwrap_or(U256::zero());
                    let value = format!("0x{:x}", value);

                    let (index, of) = index.overflowing_sub(offset);
                    assert!(!of);
                    assert!(index < len);
                    let mem_interaction = MemoryInteraction {
                        memory_type,
                        page,
                        address: index,
                        value,
                        direction: MemoryAccessType::Write
                    };

                    mem_interaction
                }
                _ => {
                    unreachable!()
                }
            };

            trace_step.memory_interactions.push(mem_interaction);
        } else {
            if aux.dst_0_reg != 0 {
                trace_step.register_interactions.insert(aux.dst_0_reg as usize, MemoryAccessType::Read);
            }
        }
        if aux.dst_1_reg != 0 {
            trace_step.register_interactions.insert(aux.dst_1_reg as usize, MemoryAccessType::Read);
        }

        self.cycle_number += 1;
    }
}