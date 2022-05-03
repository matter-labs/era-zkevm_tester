use zk_evm::aux_structures::MemoryQuery;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemoryLogWitnessTracer {
    pub is_dummy: bool,
    pub queries: Vec<MemoryQuery>,
}

use zk_evm::witness_trace::VmWitnessTracer;

impl VmWitnessTracer for MemoryLogWitnessTracer {
    fn add_memory_query(&mut self, monotonic_cycle_counter: u32, memory_query: MemoryQuery) {
        if self.is_dummy {
            return;
        }
        self.queries.push(memory_query);
    }
}
