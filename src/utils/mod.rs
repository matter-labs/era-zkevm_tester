pub trait FixedLengthIterator<'a, I: 'a, const N: usize>: Iterator<Item = I>
where
    Self: 'a,
{
    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        <Self as Iterator>::next(self)
    }
}

pub trait IntoFixedLengthIterator<I: 'static, const N: usize> {
    type IntoIter: FixedLengthIterator<'static, I, N>;
    fn into_iter(self) -> Self::IntoIter;
}

pub trait IntoFixedLengthByteIterator<const N: usize> {
    type IntoIter: FixedLengthIterator<'static, u8, N>;
    fn into_le_iter(self) -> Self::IntoIter;
    fn into_be_iter(self) -> Self::IntoIter;
}

pub struct FixedBufferValueIterator<T, const N: usize> {
    iter: std::array::IntoIter<T, N>,
}

impl<T: Clone, const N: usize> Iterator for FixedBufferValueIterator<T, N> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<T: Clone + 'static, const N: usize> FixedLengthIterator<'static, T, N>
    for FixedBufferValueIterator<T, N>
{
}

use zk_evm::{vm_state::PrimitiveValue, zkevm_opcode_defs::{fat_pointer, FatPointer}};

use crate::U256;
impl IntoFixedLengthByteIterator<32> for U256 {
    type IntoIter = FixedBufferValueIterator<u8, 32>;
    fn into_le_iter(self) -> Self::IntoIter {
        let mut buffer = [0u8; 32];
        self.to_little_endian(&mut buffer);

        FixedBufferValueIterator {
            iter: buffer.into_iter(),
        }
    }

    fn into_be_iter(self) -> Self::IntoIter {
        let mut buffer = [0u8; 32];
        self.to_big_endian(&mut buffer);

        FixedBufferValueIterator {
            iter: buffer.into_iter(),
        }
    }
}

pub(crate) fn form_initial_calldata_ptr(calldata_page: u32, calldata_length: u32) -> PrimitiveValue {
    let fat_pointer = FatPointer {
        offset: 0,
        memory_page: calldata_page,
        start: 0,
        length: calldata_length,
    };

    PrimitiveValue { value: fat_pointer.to_u256(), is_pointer: true }
}
