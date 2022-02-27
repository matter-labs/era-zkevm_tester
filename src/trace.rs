use super::*;

use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::slice::SliceIndex;

use serde::{Deserialize, Serialize};
use zk_evm::zkevm_opcode_defs::{Opcode, OpcodeVariant, REGISTERS_COUNT};

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
    memory_snapshots: Vec<MemorySnapshot>,
    error: Option<String>,
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
    stack,
    calldata,
    returndata,
    code,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum MemoryAccessType {
    Read,
    Write,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MemoryInteraction {
    memory_type: MemoryType,
    page: u32,
    address: u32,
    value: String,
    direction: MemoryAccessType,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VmTrace {
    steps: Vec<VmExecutionStep>,
    sources: HashMap<String, ContractSourceDebugInfo>,
}

use crate::default_environment::*;
use crate::runners::compiler_tests::{calldata_to_aligned_data, contract_bytecode_to_words};
use zk_evm::testing::*;

pub fn run_text_assembly_full_trace(
    assembly: String,
    calldata: Vec<u8>,
    num_cycles: usize,
) -> VmTrace {
    let vm_assembly =
        Assembly::try_from(assembly.clone()).expect("must get a valid assembly as the input");

    let empty_callstack_dummy_debug_info = ContractSourceDebugInfo {
        assembly_code: "nop r0, r0, r0, r0".to_owned(),
        pc_line_mapping: HashMap::from([(0, 0)]),
        active_lines: HashSet::from([0]),
    };

    let debug_info = ContractSourceDebugInfo {
        assembly_code: vm_assembly.assembly_code.clone(),
        pc_line_mapping: vm_assembly.pc_line_mapping.clone(),
        active_lines: HashSet::new(),
    };

    let assembly = vm_assembly.compile_to_bytecode().unwrap();

    let mut tools = create_default_testing_tools();
    let block_properties = create_default_block_properties();
    let mut vm = create_vm_with_default_settings(&mut tools, &block_properties);

    // manually encode LE
    let opcodes = contract_bytecode_to_words(assembly);
    let calldata_words = calldata_to_aligned_data(&calldata);

    // set registers r1-r4 for external call convension
    vm.local_state.registers[0] = U256::zero();
    let mut r2 = U256::zero();
    r2.0[0] = calldata.len() as u64;
    vm.local_state.registers[1] = r2;
    vm.local_state.registers[2] = U256::zero();
    vm.local_state.registers[3] = U256::zero();

    vm.memory.populate(vec![
        (ENTRY_POINT_PAGE, opcodes),
        (CALLDATA_PAGE, calldata_words),
    ]);

    let mut tracer = VmDebugTracer::new(debug_info);

    for _ in 0..num_cycles {
        vm.cycle(&mut tracer);
    }

    let VmDebugTracer {
        steps, debug_info, ..
    } = tracer;

    let mut sources = HashMap::new();
    sources.insert(DEFAULT_CALLEE_HEX.to_owned(), debug_info);
    sources.insert(
        EMPTY_CONTEXT_HEX.to_owned(),
        empty_callstack_dummy_debug_info,
    );

    let full_trace = VmTrace { steps, sources };

    full_trace
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

pub struct VmDebugTracer {
    pub debug_info: ContractSourceDebugInfo,
    regs_before: Option<[U256; REGISTERS_COUNT]>,
    aux_info: Option<AfterDecodingData>,
    callstack_info: Option<CallStackEntry>,
    cycle_number: u32,
    did_call_recently: bool,
    did_return_recently: bool,
    pub steps: Vec<VmExecutionStep>,
}

impl std::fmt::Debug for VmDebugTracer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VmDebugTracer").finish()
    }
}

impl VmDebugTracer {
    pub fn new(debug_info: ContractSourceDebugInfo) -> Self {
        Self {
            debug_info,
            regs_before: None,
            aux_info: None,
            callstack_info: None,
            did_call_recently: true,
            did_return_recently: false,
            cycle_number: 0u32,
            steps: vec![],
        }
    }
}

use zk_evm::abstractions::*;

impl zk_evm::abstractions::Tracer for VmDebugTracer {
    const CALL_BEFORE_DECODING: bool = false;
    const CALL_AFTER_DECODING: bool = true;
    const CALL_BEFORE_EXECUTION: bool = true;
    const CALL_AFTER_EXECUTION: bool = true;

    type SupportedMemory = SimpleMemory;

    fn before_decoding(&mut self, state: VmLocalStateData<'_>, memory: &Self::SupportedMemory) {}
    fn after_decoding(
        &mut self,
        state: VmLocalStateData<'_>,
        data: AfterDecodingData,
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
        let contract_address = format!("0x{:x}", current_context.this_address);
        let code_page = current_context.code_page.0;
        let base_memory_page = current_context.base_memory_page.0;
        let calldata_page = current_context.calldata_page.0;
        let returndata_page = current_context.returndata_page.0;
        self.callstack_info = Some(current_context.clone());
        drop(current_context);
        self.debug_info.active_lines.insert(current_pc as usize);
        let flags = flags_into_description(&state.vm_local_state.flags);

        self.regs_before = Some(state.vm_local_state.registers);

        let registers = state
            .vm_local_state
            .registers
            .map(|el| format!("0x{:x}", el));

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
            memory_snapshots: vec![],
            error,
        };

        // special case for initial cycle
        if self.did_call_recently {
            let calldata_offset = state.vm_local_state.registers[0].0[0] as usize;
            let calldata_length = state.vm_local_state.registers[1].0[0] as usize;
            let beginning_word = calldata_offset / 32;
            let end = calldata_offset + calldata_length;
            let mut end_word = end / 32;
            if end % 32 != 0 {
                end_word += 1;
            }

            let initial_calldata =
                memory.dump_page_content(calldata_page, (beginning_word as u32)..(end_word as u32));
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
            // get new context
            let current_context = state.vm_local_state.callstack.get_current_stack();
            let returndata_page = current_context.returndata_page.0;
            let returndata_offset = state.vm_local_state.registers[0].0[0] as usize;
            let returndata_len = state.vm_local_state.registers[1].0[0] as usize;

            let beginning_word = returndata_offset / 32;
            let end = returndata_offset + returndata_len;
            let mut end_word = end / 32;
            if end % 32 != 0 {
                end_word += 1;
            }

            let initial_returndata = memory
                .dump_page_content(returndata_page, (beginning_word as u32)..(end_word as u32));
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
        state: VmLocalStateData<'_>,
        data: BeforeExecutionData,
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
                    == CallStackEntry::heap_page_from_base(MemoryPage(base_memory_page)).0
                    || page
                        == CallStackEntry::stack_page_from_base(MemoryPage(base_memory_page)).0
                    || page == code_page =>
                {
                    let memory_type = if page
                        == CallStackEntry::heap_page_from_base(MemoryPage(base_memory_page)).0
                    {
                        assert_eq!(memory_type, zk_evm::abstractions::MemoryType::Heap);
                        MemoryType::heap
                    } else if page
                        == CallStackEntry::stack_page_from_base(MemoryPage(base_memory_page)).0
                    {
                        assert_eq!(memory_type, zk_evm::abstractions::MemoryType::Stack);
                        MemoryType::stack
                    } else if page == code_page {
                        assert_eq!(memory_type, zk_evm::abstractions::MemoryType::Code);
                        MemoryType::code
                    } else {
                        unreachable!()
                    };

                    let value = memory
                        .inner
                        .get(&page)
                        .unwrap_or(&vec![])
                        .get(index as usize)
                        .copied()
                        .unwrap_or(U256::zero());
                    let value = format!("0x{:x}", value);
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
        state: VmLocalStateData<'_>,
        data: AfterExecutionData,
        memory: &Self::SupportedMemory,
    ) {
        // let aux = self.aux_info.take().unwrap();
        let regs_before = self.regs_before.take().unwrap();
        let potentially_previous_context = self.callstack_info.take().unwrap();
        let code_page = potentially_previous_context.code_page.0;
        let base_memory_page = potentially_previous_context.base_memory_page.0;
        // - register writes
        // - memory writes

        let trace_step = self.steps.last_mut().unwrap();

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
                    == CallStackEntry::heap_page_from_base(MemoryPage(base_memory_page)).0
                    || page
                        == CallStackEntry::stack_page_from_base(MemoryPage(base_memory_page)).0
                    || page == code_page =>
                {
                    let memory_type = if page
                        == CallStackEntry::heap_page_from_base(MemoryPage(base_memory_page)).0
                    {
                        assert_eq!(memory_type, zk_evm::abstractions::MemoryType::Heap);
                        MemoryType::heap
                    } else if page
                        == CallStackEntry::stack_page_from_base(MemoryPage(base_memory_page)).0
                    {
                        assert_eq!(memory_type, zk_evm::abstractions::MemoryType::Stack);
                        MemoryType::stack
                    } else if page == code_page {
                        assert_eq!(memory_type, zk_evm::abstractions::MemoryType::Code);
                        MemoryType::code
                    } else {
                        unreachable!()
                    };

                    let value = memory
                        .inner
                        .get(&page)
                        .unwrap_or(&vec![])
                        .get(index as usize)
                        .copied()
                        .unwrap_or(U256::zero());
                    let value = format!("0x{:x}", value);
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
        if let Opcode::FarCall(far_call_variant) = data.opcode.variant.opcode {
            self.did_call_recently = true;
        }

        // special case for call or return
        if let Opcode::Ret(return_variant) = data.opcode.variant.opcode {
            if !potentially_previous_context.is_local_frame {
                // only on far return
                self.did_return_recently = true;
            }
        }

        self.cycle_number += 1;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // const SIMPLE_ASSEMBLY: &'static str = r#"
    // .rodata
    // RET_CONST:
    //     .cell 4294967296
    // .text
    // main:
    //     add 0, r0, r5
    //     add 32, r0, r6
    //     uma.calldata_read r5, r0, r5
    //     uma.calldata_read r6, r0, r6
    //     add! r5, r6, r5
    //     uma.heap_write r0, r5, r0
    //     add @RET_CONST[0], r0, r1
    //     ret.ok r1
    // "#;

    const SIMPLE_ASSEMBLY: &'static str = r#"
        .text
        .file	"Test_26"
        .rodata.cst32
        .p2align	5
    CPI0_0:
        .cell 16777184
    CPI0_1:
        .cell 16777152
    CPI0_2:
        .cell 4294967297
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
        add	@CPI0_2[0], r0, r2
        mul	r1, r2, r1, r2
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
        .cell 16777184
    CPI1_2:
        .cell -26959946667150639794667015087019630673637144422540572481103610249216
    CPI1_3:
        .cell 28023726311554802966544231341579932116438770666993405431137050659635310100480
    CPI1_4:
        .cell -4
    CPI1_5:
        .cell 40953307615929575801107647705360601464619672688377251939886941387873771847680
    CPI1_6:
        .cell 57896044618658097711785492504343953926634992332820282019728792003956564819967
        .text
    __selector:
    .func_begin1:
        add	128, r0, r1
        add	64, r0, r2
        uma.heap_write	r2, r1, r0
        add	@CPI1_0[0], r0, r2
        uma.heap_read	r2, r0, r2
        add	3, r0, r3
        sub!	r2, r3, r3
        jump.gt	@.BB1_3
        jump	@.BB1_1
    .BB1_3:
        add	@CPI1_1[0], r0, r3
        uma.heap_read	r3, r0, r3
        uma.calldata_read	r3, r0, r3
        and	@CPI1_2[0], r3, r3
        add	@CPI1_3[0], r0, r4
        sub!	r3, r4, r4
        add	@CPI1_4[0], r2, r3
        add	42, r0, r2
        add	@CPI1_6[0], r0, r4
        sub!	r3, r4, r3
        jump.le	@.BB1_2
        jump	@.BB1_1
    .BB1_5:
        add	@CPI1_4[0], r0, r4
        add	@CPI1_5[0], r0, r5
        sub!	r3, r5, r3
        add	r2, r4, r3
        add	99, r0, r2
        add	@CPI1_6[0], r0, r4
        sub!	r3, r4, r3
    .BB1_2:
        uma.heap_write	r1, r2, r0
        add	@CPI1_1[0], r0, r2
        uma.heap_write	r2, r1, r0
        add	32, r0, r1
        add	@CPI1_0[0], r0, r2
        uma.heap_write	r2, r1, r0
        ret
    .BB1_1:
        add	0, r0, r1
        add	@CPI1_1[0], r0, r2
        uma.heap_write	r2, r1, r0
        add	@CPI1_0[0], r0, r2
        uma.heap_write	r2, r1, r0
        ret.panic r0
    .func_end1:

        .note.GNU-stack
    "#;

    #[test]
    fn run_something() {
        use super::*;

        let mut input = vec![0u8; 64];
        input[31] = 1;
        input[63] = 2;

        let trace = run_text_assembly_full_trace(SIMPLE_ASSEMBLY.to_owned(), input, 12);

        let _ = std::fs::remove_file("tmp.json");
        let mut file = std::fs::File::create("tmp.json").unwrap();
        let json = serde_json::to_string(&trace).unwrap();

        file.write_all(json.as_bytes()).unwrap();
    }

    #[test]
    fn test_manually() {
        use crate::runners::compiler_tests::*;

        use futures::executor::block_on;
        set_debug(true);

        let assembly = Assembly::try_from(SIMPLE_ASSEMBLY.to_owned()).unwrap();
        let calldata = hex::decode("5a8ac02d").unwrap();
        let snapshot = block_on(run_vm(
            assembly.clone(),
            calldata,
            HashMap::new(),
            vec![],
            None,
            VmLaunchOption::Default,
            1024,
            u16::MAX as usize,
            vec![assembly.clone()],
            vec![],
            HashMap::new(),
        ));

        let VmSnapshot {
            registers,
            flags,
            timestamp,
            memory_page_counter,
            tx_number_in_block,
            previous_pc,
            did_call_or_ret_recently,
            tx_origin,
            calldata_area_dump,
            returndata_area_dump,
            execution_has_ended,
            stack_dump,
            heap_dump,
            storage,
            deployed_contracts,
            execution_result,
            returndata_bytes,
        } = snapshot;
        dbg!(execution_has_ended);
        dbg!(execution_result);
        dbg!(registers);
        dbg!(timestamp);
        dbg!(hex::encode((&returndata_bytes)));
    }
}
