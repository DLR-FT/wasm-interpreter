use crate::{core::utils::ToUsizeExt, Limits, MemType, RuntimeError};

pub mod linear_memory;

pub struct MemInst {
    pub ty: MemType,
    pub mem: linear_memory::LinearMemory,
}
impl core::fmt::Debug for MemInst {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MemInst")
            .field("ty", &self.ty)
            .finish_non_exhaustive()
    }
}

impl MemInst {
    /// If the grow is successful, the previous length of the memory is returned in pages.
    ///
    /// <https://webassembly.github.io/spec/core/exec/modules.html#growing-memories>
    pub fn grow(&mut self, n: u32) -> Result<u32, RuntimeError> {
        // step 2
        let len_pages = self.len_pages();

        // step 3,4
        let Some(new_len_pages) = len_pages.checked_add(n) else {
            return Err(RuntimeError::MemoryGrowOverflowed);
        };

        // We limit the number of pages to 2^16 because otherwise addresses for the next page's
        // contents would exceed 2^32 - 1.
        if new_len_pages >= 2u32.pow(16) {
            return Err(RuntimeError::MemoryGrowOverflowed);
        }

        // roughly matches step 5, 6, 7
        // checks limits_prime.valid() for limits_prime := { min: len, max: self.ty.lim.max }
        // https://webassembly.github.io/spec/core/valid/types.html#limits
        if self.ty.limits.max.is_some_and(|max| new_len_pages > max) {
            return Err(RuntimeError::MemoryGrowExceededLimit);
        }

        let limits_prime = Limits {
            min: new_len_pages,
            max: self.ty.limits.max,
        };

        // step 8
        self.mem.grow(n.into_usize());

        // step 9
        self.ty.limits = limits_prime;

        Ok(len_pages)
    }

    /// The length of this memory in pages.
    pub fn len_pages(&self) -> u32 {
        u32::try_from(self.mem.len_pages()).expect("num pages is not greater or equal to 2^32 because that would exceed even the amount of bytes in the memory")
    }
}
