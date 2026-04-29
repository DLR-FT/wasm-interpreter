use alloc::vec::Vec;

use crate::{
    linear_memory::PageCountTy, little_endian::LittleEndianBytes, RuntimeError, TrapError,
};

pub struct UnsharedLinearMemory<const PAGE_SIZE: usize = { crate::Limits::MEM_PAGE_SIZE as usize }>
{
    data: Vec<u8>,
}

impl<const PAGE_SIZE: usize> UnsharedLinearMemory<PAGE_SIZE> {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn new_with_initial_pages(pages: PageCountTy) -> Self {
        let size_bytes = PAGE_SIZE * usize::from(pages);
        let mut data = Vec::with_capacity(size_bytes);
        data.resize(size_bytes, 0);

        Self { data }
    }

    pub fn grow(&mut self, pages_to_add: PageCountTy) {
        let prior_length_bytes = self.data.len();
        let new_length_bytes = prior_length_bytes + PAGE_SIZE * usize::from(pages_to_add);
        self.data.resize(new_length_bytes, 0);
    }

    pub fn pages(&self) -> PageCountTy {
        PageCountTy::try_from(self.data.len() / PAGE_SIZE).unwrap()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn store<const N: usize, T: LittleEndianBytes<N>>(
        &mut self,
        index: usize,
        value: T,
    ) -> Result<(), RuntimeError> {
        self.store_bytes::<N>(index, value.to_le_bytes())
    }

    pub fn store_bytes<const N: usize>(
        &mut self,
        index: usize,
        bytes: [u8; N],
    ) -> Result<(), RuntimeError> {
        /* check destination for out of bounds access */
        // A value must fit into the linear memory
        if N > self.data.len() {
            error!("value does not fit into linear memory");
            return Err(TrapError::MemoryOrDataAccessOutOfBounds.into());
        }

        // The following statement must be true
        // `index + N <= data.len()`
        // This check verifies it, while avoiding the possible overflow. The subtraction can not
        // underflow because of the previous check.

        if index > self.data.len() - N {
            error!("value write would extend beyond the end of the linear memory");
            return Err(TrapError::MemoryOrDataAccessOutOfBounds.into());
        }

        /* do the store */
        for (i, byte) in bytes.into_iter().enumerate() {
            // SAFETY:
            // The safety of this `unsafe` block depends on the index being valid, which it is
            // because:
            //
            // - the first if statement in this function guarantees that a `T` can fit into the
            //   `LinearMemory` `&mut self`
            // - the second if statement in this function guarantees that even with the offset
            //   `index`, writing all of `value`'s bytes does not extend beyond the last byte in
            //   the `LinearMemory` `&mut self`
            let dst = unsafe { self.data.get_unchecked_mut(i + index) };
            *dst = byte;
        }

        Ok(())
    }

    pub fn load<const N: usize, T: LittleEndianBytes<N>>(
        &self,
        index: usize,
    ) -> Result<T, RuntimeError> {
        self.load_bytes::<N>(index).map(T::from_le_bytes)
    }

    pub fn load_bytes<const N: usize>(&self, index: usize) -> Result<[u8; N], RuntimeError> {
        /* check source for out of bounds access */
        // A value must fit into the linear memory
        if N > self.data.len() {
            error!("value does not fit into linear memory");
            return Err(TrapError::MemoryOrDataAccessOutOfBounds.into());
        }

        // The following statement must be true
        // `index + N <= data.len()`
        // This check verifies it, while avoiding the possible overflow. The subtraction can not
        // underflow because of the previous assert.

        if index > self.data.len() - N {
            error!("value read would extend beyond the end of the linear_memory");
            return Err(TrapError::MemoryOrDataAccessOutOfBounds.into());
        }

        let mut bytes = [0; N];

        /* do the load */
        for (i, byte) in bytes.iter_mut().enumerate() {
            // SAFETY:
            // The safety of this `unsafe` block depends on the index being valid, which it is
            // because:
            //
            // - the first if statement in this function guarantees that a `T` can fit into the
            //   `LinearMemory` `&self`
            // - the second if statement in this function guarantees that even with the offset
            //   `index`, reading all `N` bytes does not extend beyond the last byte in
            //   the `LinearMemory` `&self`
            let src = unsafe { self.data.get_unchecked(i + index) };
            *byte = *src;
        }

        Ok(bytes)
    }

    pub fn fill(&mut self, index: usize, data_byte: u8, count: usize) -> Result<(), RuntimeError> {
        /* check destination for out of bounds access */
        // Specification step 12.
        if count > self.data.len() {
            error!("fill count is bigger than the linear memory");
            return Err(TrapError::MemoryOrDataAccessOutOfBounds.into());
        }

        // Specification step 12.
        if index > self.data.len() - count {
            error!("fill extends beyond the linear memory's end");
            return Err(TrapError::MemoryOrDataAccessOutOfBounds.into());
        }

        /* check if there is anything to be done */
        // Specification step 13.
        if count == 0 {
            return Ok(());
        }

        /* do the fill */
        // Specification step 14-21.
        for i in index..(index + count) {
            // SAFETY:
            // The safety of this `unsafe` block depends on the index being valid, which it is
            // because:
            //
            // - the first if statement in this function guarantees that `count` elements can fit
            //   into the `LinearMemory` `&self`
            // - the second if statement in this function guarantees that even with the offset
            //   `index`, writing all `count`'s bytes does not extend beyond the last byte in
            //   the `LinearMemory` `&self`
            let lin_mem_byte = unsafe { self.data.get_unchecked_mut(i) };
            *lin_mem_byte = data_byte;
        }

        Ok(())
    }

    pub fn copy(
        &mut self,
        destination_index: usize,
        source_mem: &Self,
        source_index: usize,
        count: usize,
    ) -> Result<(), RuntimeError> {
        /* check source for out of bounds access */
        // Specification step 12.
        if count > source_mem.data.len() {
            error!("copy count is bigger than the source linear memory");
            return Err(TrapError::MemoryOrDataAccessOutOfBounds.into());
        }

        // Specification step 12.
        if source_index > source_mem.data.len() - count {
            error!("copy source extends beyond the linear memory's end");
            return Err(TrapError::MemoryOrDataAccessOutOfBounds.into());
        }

        /* check destination for out of bounds access */
        // Specification step 12.
        if count > self.data.len() {
            error!("copy count is bigger than the destination linear memory");
            return Err(TrapError::MemoryOrDataAccessOutOfBounds.into());
        }

        // Specification step 12.
        if destination_index > self.data.len() - count {
            error!("copy destination extends beyond the linear memory's end");
            return Err(TrapError::MemoryOrDataAccessOutOfBounds.into());
        }

        /* check if there is anything to be done */
        // Specification step 13.
        if count == 0 {
            return Ok(());
        }

        /* do the copy */
        let copy_one_byte = move |i| {
            // SAFETY:
            // The safety of this `unsafe` block depends on the index being valid, which it is
            // because:
            //
            // - the first if statement in this function guarantees that `count` elements can fit
            //   into the `LinearMemory` `&source_mem`
            // - the second if statement in this function guarantees that even with the offset
            //   `source_index`, writing all `count`'s bytes does not extend beyond the last byte in
            let src_byte: &u8 = unsafe { source_mem.data.get_unchecked(i + source_index) };

            // SAFETY:
            // The safety of this `unsafe` block depends on the index being valid, which it is
            // because:
            //
            // - the third if statement in this function guarantees that `count` elements can fit
            //   into the `LinearMemory` `&self`
            // - the fourth if statement in this function guarantees that even with the offset
            //   `destination_index`, writing all `count`'s bytes does not extend beyond the last byte in
            //   the `LinearMemory` `&self`
            let dst_byte: &mut u8 = unsafe { self.data.get_unchecked_mut(i + destination_index) };

            *dst_byte = *src_byte
        };

        // TODO investigate if it is worth to only do reverse order copy if there is actual overlap

        // Specification step 14.
        if destination_index <= source_index {
            // if source index is bigger than or equal to destination index, forward processing copy
            // handles overlaps just fine
            (0..count).for_each(copy_one_byte)
        }
        // Specification step 15.
        else {
            // if source index is smaller than destination index, backward processing is required to
            // avoid data loss on overlaps
            (0..count).rev().for_each(copy_one_byte)
        }

        Ok(())
    }

    pub fn init(
        &mut self,
        destination_index: usize,
        source_data: &[u8],
        source_index: usize,
        count: usize,
    ) -> Result<(), RuntimeError> {
        let data_len = self.data.len();

        /* check source for out of bounds access */
        // Specification step 16.
        if count > data_len {
            error!("init count is bigger than the data instance");
            return Err(TrapError::MemoryOrDataAccessOutOfBounds.into());
        }

        // Specification step 16.
        if source_index > data_len - count {
            error!("init source extends beyond the data instance's end");
            return Err(TrapError::MemoryOrDataAccessOutOfBounds.into());
        }

        /* check destination for out of bounds access */
        // Specification step 16.
        if count > self.data.len() {
            error!("init count is bigger than the linear memory");
            return Err(TrapError::MemoryOrDataAccessOutOfBounds.into());
        }

        // Specification step 16.
        if destination_index > self.data.len() - count {
            error!("init extends beyond the linear memory's end");
            return Err(TrapError::MemoryOrDataAccessOutOfBounds.into());
        }

        /* check if there is anything to be done */
        // Specification step 17.
        if count == 0 {
            return Ok(());
        }

        /* do the init */
        // Specification step 18-27.
        for i in 0..count {
            // SAFETY:
            // The safety of this `unsafe` block depends on the index being valid, which it is
            // because:
            //
            // - the first if statement in this function guarantees that `count` elements can fit
            //   into the `LinearMemory` `&source_mem`
            // - the second if statement in this function guarantees that even with the offset
            //   `source_index`, writing all `count`'s bytes does not extend beyond the last byte in
            let src_byte = unsafe { source_data.get_unchecked(i + source_index) };

            // SAFETY:
            // The safety of this `unsafe` block depends on the index being valid, which it is
            // because:
            //
            // - the third if statement in this function guarantees that `count` elements can fit
            //   into the `LinearMemory` `&self`
            // - the fourth if statement in this function guarantees that even with the offset
            //   `destination_index`, writing all `count`'s bytes does not extend beyond the last byte in
            //   the `LinearMemory` `&self`
            let dst_byte = unsafe { self.data.get_unchecked_mut(i + destination_index) };
            *dst_byte = *src_byte;
        }

        Ok(())
    }

    /// Allows a given closure to temporarily access the entire memory as a `&mut [u8]`.
    pub fn access_mut_slice<R>(&mut self, accessor: impl FnOnce(&mut [u8]) -> R) -> R {
        accessor(&mut self.data)
    }
}

impl<const PAGE_SIZE: usize> core::fmt::Debug for UnsharedLinearMemory<PAGE_SIZE> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // TODO
        f.debug_struct("UnsharedLinearMemory")
            .finish_non_exhaustive()
    }
}

impl<const PAGE_SIZE: usize> Default for UnsharedLinearMemory<PAGE_SIZE> {
    fn default() -> Self {
        Self::new()
    }
}
