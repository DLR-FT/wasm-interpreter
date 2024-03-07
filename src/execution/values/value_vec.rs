use alloc::boxed::Box;
use alloc::vec::Vec;
use core::iter;
use core::ops::{Index, IndexMut, Range};

/// A vector with variably sized elements. Each element is a byte slice of fixed size.
/// Also the vector's length is fixed and zero-initialized.
pub(crate) struct VarSizeVec {
    inner: Box<[u8]>,
    ranges: Box<[Range<usize>]>,
}

impl VarSizeVec {
    pub fn new(type_sizes: impl Iterator<Item = usize>) -> Self {
        let mut inner = Vec::new();
        let mut ranges = Vec::new();

        let mut cur_size = 0;
        for size in type_sizes {
            ranges.push(cur_size..(cur_size + size));
            inner.extend(iter::repeat(0).take(size));
            cur_size += size;
        }

        Self {
            inner: inner.into_boxed_slice(),
            ranges: ranges.into_boxed_slice(),
        }
    }

    pub fn get(&self, idx: usize) -> Option<&[u8]> {
        let range = self.ranges.get(idx)?.clone();
        Some(&self.inner[range])
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut [u8]> {
        let range = self.ranges.get(idx)?.clone();
        Some(&mut self.inner[range])
    }
}

impl Index<usize> for VarSizeVec {
    type Output = [u8];

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).unwrap()
    }
}
impl IndexMut<usize> for VarSizeVec {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}
