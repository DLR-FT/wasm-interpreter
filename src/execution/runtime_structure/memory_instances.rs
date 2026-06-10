use crate::{
    core::utils::ToUsizeExt, execution::store::linear_memory::LinearMemory, Limits, MemType,
    RuntimeError, TrapError,
};

pub struct MemInst {
    pub ty: MemType,
    pub mem: LinearMemory,
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
    pub fn grow(&mut self, n: u32) -> Result<(), RuntimeError> {
        // TODO refactor error, the spec Table.grow raises Memory.{SizeOverflow, SizeLimit, OutOfMemory}
        let len = n + self.mem.pages() as u32;
        if len > Limits::MAX_MEM_PAGES {
            return Err(TrapError::MemoryOrDataAccessOutOfBounds.into());
        }

        // roughly matches step 4,5,6
        // checks limits_prime.valid() for limits_prime := { min: len, max: self.ty.lim.max }
        // https://webassembly.github.io/spec/core/valid/types.html#limits
        if self.ty.limits.max.map(|max| len > max).unwrap_or(false) {
            return Err(TrapError::MemoryOrDataAccessOutOfBounds.into());
        }
        let limits_prime = Limits {
            min: len,
            max: self.ty.limits.max,
        };

        self.mem.grow(n.try_into().unwrap());

        self.ty.limits = limits_prime;
        Ok(())
    }

    /// Can never be bigger than 65,356 pages
    pub fn size(&self) -> usize {
        self.mem.len() / (crate::Limits::MEM_PAGE_SIZE.into_usize())
    }
}
