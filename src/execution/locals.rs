use alloc::boxed::Box;
use alloc::vec::Vec;
use core::iter;
use core::ops::Range;

use crate::execution::unwrap_validated::UnwrapValidatedExt;

/// A helper for managing values of locals (and parameters) during function execution.
pub struct Locals {
    data: Box<[u8]>,
    element_ranges: Box<[Range<usize>]>,
}

impl Locals {
    pub fn new<'a>(
        parameters: impl Iterator<Item = &'a [u8]>,
        local_sizes: impl Iterator<Item = usize>,
    ) -> Self {
        let mut data = Vec::new();
        let mut element_ranges = Vec::new();
        let mut cur_size = 0;

        // Copy parameters' data
        for parameter in parameters {
            data.extend(&*parameter);
            element_ranges.push(cur_size..(cur_size + parameter.len()));
            cur_size += parameter.len();
        }

        // Zero-initialize remaining locals with given sizes
        for local_size in local_sizes {
            data.extend(iter::repeat(0).take(local_size));
            element_ranges.push(cur_size..(cur_size + local_size));
            cur_size += local_size;
        }

        Self {
            data: data.into_boxed_slice(),
            element_ranges: element_ranges.into_boxed_slice(),
        }
    }

    pub fn get(&self, idx: usize) -> &[u8] {
        self.data
            .get(self.get_element_range(idx))
            .unwrap_validated()
    }

    pub fn get_mut(&mut self, idx: usize) -> &mut [u8] {
        self.data
            .get_mut(self.get_element_range(idx))
            .unwrap_validated()
    }

    fn get_element_range(&self, idx: usize) -> Range<usize> {
        self.element_ranges.get(idx).unwrap_validated().clone()
    }
}
