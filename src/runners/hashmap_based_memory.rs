use crate::U256;
use std::collections::HashMap;
use zk_evm::abstractions::MEMORY_CELLS_OTHER_PAGES;

#[derive(Debug)]
pub struct SimpleHashmapMemory {
    pub inner: HashMap<u32, HashMap<u32, PrimitiveValue>>,
}

// as usual, if we rollback the current frame then we apply changes to storage immediately,
// otherwise we carry rollbacks to the parent's frames

impl SimpleHashmapMemory {
    pub fn new() -> Self {
        Self {
            inner: HashMap::default(),
        }
    }

    pub fn populate(&mut self, elements: Vec<(u32, Vec<U256>)>) -> Vec<(u32, usize)> {
        let mut results = vec![];
        for (page, values) in elements.into_iter() {
            assert!(!self.inner.contains_key(&page));
            let len = values.len();
            assert!(len <= MEMORY_CELLS_OTHER_PAGES);
            let mut inner_map = HashMap::with_capacity(len);
            for (index, value) in values.into_iter().enumerate() {
                let value = PrimitiveValue::from_value(value);
                inner_map.insert(index as u32, value);
            }
            self.inner.insert(page, inner_map);
            results.push((page, len));
        }

        results
    }

    pub fn dump_page_content(
        &self,
        page_number: u32,
        range: std::ops::Range<u32>,
    ) -> Vec<[u8; 32]> {
        let as_u256 = self.dump_page_content_as_u256_words(page_number, range);
        let mut result = Vec::with_capacity(as_u256.len());
        let mut buffer = [0u8; 32];
        for word in as_u256.into_iter() {
            word.to_big_endian(&mut buffer);
            result.push(buffer);
        }

        result
    }

    pub fn dump_page_content_as_u256_words(
        &self,
        page_number: u32,
        range: std::ops::Range<u32>,
    ) -> Vec<U256> {
        if let Some(page) = self.inner.get(&page_number) {
            let mut result = Vec::with_capacity(range.len() as usize);
            for i in range {
                if let Some(word) = page.get(&i) {
                    result.push(word.value);
                } else {
                    result.push(U256::zero());
                }
            }

            result
        } else {
            vec![U256::zero(); range.len()]
        }
    }

    pub fn dump_full_page_as_u256_words(&self, page_number: u32) -> Vec<U256> {
        if let Some(page) = self.inner.get(&page_number) {
            let max_key = *page.keys().max().unwrap_or(&0);
            let mut result = Vec::with_capacity(max_key as usize);
            for key in 0..max_key {
                let word = page.get(&key).map(|el| el.value).unwrap_or(U256::zero());
                result.push(word);
            }

            result
        } else {
            vec![]
        }
    }

    pub fn dump_full_page(&self, page_number: u32) -> Vec<[u8; 32]> {
        let as_u256 = self.dump_full_page_as_u256_words(page_number);
        let mut result = Vec::with_capacity(as_u256.len());
        let mut buffer = [0u8; 32];
        for word in as_u256.into_iter() {
            word.to_big_endian(&mut buffer);
            result.push(buffer);
        }

        result
    }
}

use zk_evm::abstractions::Memory;
use zk_evm::aux_structures::MemoryQuery;
use zk_evm::vm_state::PrimitiveValue;

impl Memory for SimpleHashmapMemory {
    fn read_code_query(&self, _monotonic_cycle_counter: u32, query: MemoryQuery) -> MemoryQuery {
        assert!(query.rw_flag == false);

        if let Some(existing) = self.inner.get(&query.location.page.0) {
            if let Some(value) = existing.get(&query.location.index.0) {
                let mut query = query;
                query.value_is_pointer = value.is_pointer;
                query.value = value.value;

                query
            } else {
                let mut query = query;
                query.value_is_pointer = false;
                query.value = U256::zero();

                query
            }
        } else {
            let mut query = query;
            query.value_is_pointer = false;
            query.value = U256::zero();

            query
        }
    }

    fn execute_partial_query(
        &mut self,
        _monotonic_cycle_counter: u32,
        mut query: MemoryQuery,
    ) -> MemoryQuery {
        let entry = self
            .inner
            .entry(query.location.page.0)
            .or_insert(HashMap::new());
        let value = entry
            .entry(query.location.index.0)
            .or_insert(PrimitiveValue::empty());
        if query.rw_flag {
            value.value = query.value;
            value.is_pointer = query.value_is_pointer;
        } else {
            query.value = value.value;
            query.value_is_pointer = value.is_pointer;
        }

        query
    }

    fn specialized_code_query(
        &mut self,
        monotonic_cycle_counter: u32,
        query: MemoryQuery,
    ) -> MemoryQuery {
        self.execute_partial_query(monotonic_cycle_counter, query)
    }
}
