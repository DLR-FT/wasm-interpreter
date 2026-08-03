use crate::{
    core::utils::ToUsizeExt,
    execution::runtime_structure::memory_instances::linear_memory::MemorySizeOverflow, Config,
    Limits, MemType, RuntimeError,
};

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
    /// See: [WebAssembly Specification 2.0 - 4.5.3.9 Growing Memories](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#growing-memories%E2%91%A0)
    pub fn grow<T: Config>(&mut self, n: u32) -> Result<u32, RuntimeError> {
        // 1. Let meminst be the memory instance to grow and n the number of pages by which to grow
        //    it.
        //
        // `self` is the meminst and `n` the number of pages by which to grow it.

        // 2. Assert: The length of meminst.data is divisible by the page size 64Ki.
        // 3. Let len be n added to the length of meminst.data divided by the page size 64Ki.
        let previous_len = self.len_pages();
        let len = u32::from(previous_len).checked_add(n);

        // 4. If len is larger than 2^16, then fail.
        //
        // Checks if it would have been larger than 2^64
        let Some(len) = len else {
            return Err(RuntimeError::MemoryOverflowed);
        };
        // Check if it is larger than 2^16
        let len = u16::try_from(len).map_err(|_| RuntimeError::MemoryOverflowed)?;

        // 5. Let limits be the structure of memory type meminst.type.
        let limits = self.ty.limits;

        // 6. Let limits' be limits with min updated to len.
        let limits_prime = Limits {
            min: u32::from(len),
            ..limits
        };

        // 7. If limits' is not valid, then fail.
        if limits_prime.max.is_some_and(|max| limits_prime.min > max) {
            return Err(RuntimeError::MemoryGrowExceededLimit);
        }

        // 8. Append n times 64Ki bytes with value 0x00 to meminst.data.
        //
        // For us this operation is fallible, as a custom upper size limit can be set through
        // `Config`.
        self.mem
            .grow::<T>(n.into_usize())
            .map_err(|MemorySizeOverflow| RuntimeError::MemoryOverflowed)?;

        // 9. Set meminst.type to the memory type limits'.
        self.ty.limits = limits_prime;

        // Additionally, return the previous length
        Ok(u32::from(previous_len))
    }

    pub fn len_pages(&self) -> u16 {
        self.mem
            .len_pages()
            .try_into()
            .expect("memory length is pages is never 2^16 or larger")
    }
}
