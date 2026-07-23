use crate::{Limits, MemType, RuntimeError};

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
    /// <https://webassembly.github.io/spec/core/exec/modules.html#growing-memories>
    pub fn grow(&mut self, n: u16) -> Result<(), RuntimeError> {
        // step 2
        let len_pages = self.len_pages();

        // step 3,4
        let Some(new_len_pages) = len_pages.checked_add(n) else {
            return Err(RuntimeError::MemoryGrowOverflowed);
        };

        // roughly matches step 5, 6, 7
        // checks limits_prime.valid() for limits_prime := { min: len, max: self.ty.lim.max }
        // https://webassembly.github.io/spec/core/valid/types.html#limits
        if self
            .ty
            .limits
            .max
            .is_some_and(|max| u32::from(new_len_pages) > max)
        {
            return Err(RuntimeError::MemoryGrowExceededLimit);
        }
        let limits_prime = Limits {
            min: u32::from(new_len_pages),
            max: self.ty.limits.max,
        };

        // step 8
        self.mem.grow(usize::from(n));

        // step 9
        self.ty.limits = limits_prime;
        Ok(())
    }

    /// The length of this memory in pages.
    pub fn len_pages(&self) -> u16 {
        self.mem.len_pages().try_into().expect(
            "we always use the default page size thereby limiting the number of pages to < 2^16",
        )
    }
}
