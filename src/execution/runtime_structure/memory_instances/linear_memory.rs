use core::{iter, num::NonZeroUsize};

use crate::{
    core::fixed_capacity_vec::FixedCapacityVec,
    execution::numerics::representations::LittleEndianBytes, Config, RuntimeError, TrapError,
};

/// A linear memory is the backing data structure for a memory instance[^memory-instances].
///
/// It is a wrapper around a contiguous, but growable sequence of bytes. Also it provides generic
/// methods to be build upon by memory instructions[^memory-instructions].
///
/// TODO: Write section on why some memory instructions are implemented here vs. implemented in
/// their instruction handlers. Also find out how we can reference the specification steps for each
/// instruction.
///
///
/// [^memory-instances]: [WebAssembly Specification 2.0 - 4.2.9. Memory Instances](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#memory-instances%E2%91%A0).
/// [^memory-instructions]: [WebAssembly Specification 2.0 - 4.4.7. Memory Instructions](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#memory-instructions%E2%91%A4).
pub struct LinearMemory {
    // Length and capacity of this vector must always be multiples of `self.page_size`.
    pub(crate) data: FixedCapacityVec<u8>,
    page_size: NonZeroUsize,
}

impl LinearMemory {
    /// Creates a new zero-initialized linear memory. Its size is determined by the given page size
    /// and number of pages.
    pub fn new_with_initial_pages<T: Config>(
        page_size: NonZeroUsize,
        pages: usize,
        upper_limit_in_pages: Option<usize>,
    ) -> Result<Self, RuntimeError> {
        // Size in bytes must not overflow
        let size_in_bytes = pages
            .checked_mul(page_size.get())
            .ok_or(RuntimeError::MemoryOverflowed)?;

        // The user may refuse this allocation or increase the number of pages we allocate.
        let allocation_size_in_pages =
            T::memory_requested_allocation(None, pages, upper_limit_in_pages)
                .ok_or(RuntimeError::HostRefusedAllocation)?;

        // If the user chose an size lower than the required size, fail early.
        if allocation_size_in_pages < pages {
            return Err(RuntimeError::MemoryOverflowed);
        }

        // New size in bytes also must not overflow
        let allocation_size_in_bytes = allocation_size_in_pages
            .checked_mul(page_size.get())
            .ok_or(RuntimeError::MemoryOverflowed)?;

        let mut data = FixedCapacityVec::with_capacity(allocation_size_in_bytes);

        // TODO optimize
        for _ in 0..size_in_bytes {
            data.push(0)
                .expect("size_in_bytes is less or equal to capacity");
        }

        Ok(Self { data, page_size })
    }

    /// Grows the current linear memory by a number of pages.
    ///
    /// This function does not check if the given upper limit is exceeded. It is only required for
    /// calling the user's configuration code.
    pub fn grow<T: Config>(
        &mut self,
        pages_to_add: usize,
        upper_limit_in_pages: Option<usize>,
    ) -> Result<(), RuntimeError> {
        // Size in pages must not overflow
        let new_len_pages = self
            .len_pages()
            .checked_add(pages_to_add)
            .ok_or(RuntimeError::MemoryOverflowed)?;

        // Size in pages must not overflow as well
        let new_len = new_len_pages
            .checked_mul(self.page_size.get())
            .ok_or(RuntimeError::MemoryOverflowed)?;

        // If the capacity does not suffice, call the user to ask for a reallocation.
        let capacity_pages = self.capacity_pages();
        if let Some(additional_capacity_pages) = new_len_pages.checked_sub(capacity_pages) {
            let num_new_pages = T::memory_requested_allocation(
                Some(capacity_pages),
                additional_capacity_pages,
                upper_limit_in_pages,
            )
            .ok_or(RuntimeError::HostRefusedAllocation)?;

            // If the user chose a reallocation size lower than the required additional capacity, it
            // is okay to return early and perform no realloc at all.
            if num_new_pages < additional_capacity_pages {
                return Err(RuntimeError::MemoryOverflowed);
            }

            // New number of bytes to allocate must not overflow
            let num_new_bytes = num_new_pages
                .checked_mul(self.page_size.get())
                .ok_or(RuntimeError::MemoryOverflowed)?;

            if self.capacity().checked_add(num_new_bytes).is_none() {
                return Err(RuntimeError::MemoryOverflowed);
            }

            // SAFETY: The final capacity does not overflow.
            unsafe { self.data.extend_reserve_unchecked(num_new_bytes) };
        }

        // TODO optimize
        let num_bytes_to_push = new_len
            .checked_sub(self.len())
            .expect("new length is never less than previous length");
        for _ in 0..num_bytes_to_push {
            self.data
                .push(0)
                .expect("the capacity check ensures that enough capacity is available");
        }

        Ok(())
    }

    /// Returns the size of this linear memory in pages.
    pub fn len_pages(&self) -> usize {
        self.len() / self.page_size()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// The size of each page in this linear memory in bytes.
    pub fn page_size(&self) -> NonZeroUsize {
        self.page_size
    }

    // Returns the capacity of the current allocation backing the linear memory.
    fn capacity(&self) -> usize {
        self.data.capacity()
    }

    // Returns the capacity of the current allocation backing the linear memory in pages.
    fn capacity_pages(&self) -> usize {
        self.data.capacity() / self.page_size()
    }

    /// Stores a `T` starting from the given index into this linear memory.
    ///
    /// The `T` must be convertible to its little-endian byte form (as required by the
    /// [`LittleEndianBytes`] bound).
    ///
    /// # Errors
    ///
    /// Same as [`LinearMemory::store_bytes`]
    pub fn store<const N: usize, T: LittleEndianBytes<N>>(
        &mut self,
        index: usize,
        value: T,
    ) -> Result<(), TrapError> {
        self.store_bytes::<N>(index, value.to_le_bytes())
    }

    /// Stores a number of bytes into this linear memory starting at the given index.
    ///
    /// # Errors
    ///
    /// - [`TrapError::MemoryOrDataAccessOutOfBounds`]: The store would have been out of bounds. The
    ///   memory remains unchanged.
    pub fn store_bytes<const N: usize>(
        &mut self,
        index: usize,
        bytes: [u8; N],
    ) -> Result<(), TrapError> {
        let end_index = index
            .checked_add(N)
            .ok_or(TrapError::MemoryOrDataAccessOutOfBounds)?;

        let target_range = index..end_index;

        let target_bytes = self
            .data
            .get_mut(target_range)
            .ok_or(TrapError::MemoryOrDataAccessOutOfBounds)?;

        target_bytes.copy_from_slice(&bytes);

        Ok(())
    }

    /// Loads a `T` starting from the given index into this linear memory.
    ///
    /// The `T` must be convertible from its little-endian byte form (as required by the
    /// [`LittleEndianBytes`] bound).
    ///
    /// # Errors
    ///
    /// Same as [`LinearMemory::load_bytes`]
    pub fn load<const N: usize, T: LittleEndianBytes<N>>(
        &self,
        index: usize,
    ) -> Result<T, TrapError> {
        self.load_bytes::<N>(index).map(T::from_le_bytes)
    }

    /// Loads a number of bytes from this linear memory starting at the given index.
    ///
    /// # Errors
    ///
    /// - [`TrapError::MemoryOrDataAccessOutOfBounds`]: The load would have been out of bounds.
    pub fn load_bytes<const N: usize>(&self, index: usize) -> Result<[u8; N], TrapError> {
        let end_index = index
            .checked_add(N)
            .ok_or(TrapError::MemoryOrDataAccessOutOfBounds)?;

        let target_range = index..end_index;

        let target_bytes = self
            .data
            .get(target_range)
            .ok_or(TrapError::MemoryOrDataAccessOutOfBounds)?;

        let bytes: [u8; N] = target_bytes
            .try_into()
            .expect("bytes slice to be of length N");

        Ok(bytes)
    }

    /// Sets a number of bytes in this linear memory to a constant `data_byte`. A total of `count`
    /// bytes are written, starting at the given index.
    ///
    /// # Errors
    ///
    /// - [`TrapError::MemoryOrDataAccessOutOfBounds`]: The fill operation would have been out of
    ///   bounds. The memory remains unchanged.
    pub fn fill(&mut self, index: usize, data_byte: u8, count: usize) -> Result<(), TrapError> {
        let end_index = index
            .checked_add(count)
            .ok_or(TrapError::MemoryOrDataAccessOutOfBounds)?;

        let target_range = index..end_index;

        /* check destination for out of bounds access */
        // Specification step 12.
        let target_bytes = self
            .data
            .get_mut(target_range)
            .ok_or(TrapError::MemoryOrDataAccessOutOfBounds)?;

        /* do the fill, do nothing if count was zero */
        // Specification step 13-21.
        target_bytes.fill(data_byte);

        Ok(())
    }

    /// Copies a number of bytes inside this linear memory from one to another location. Both the
    /// source and destination may overlap.
    ///
    /// A total of `count` bytes will be copied, starting at the given source and destination
    /// indices.
    ///
    /// # Errors
    ///
    /// - [`TrapError::MemoryOrDataAccessOutOfBounds`]: The copy operation would have been out of
    ///   bounds. The memory remains unchanged.
    pub fn copy_within(
        &mut self,
        destination_index: usize,
        source_index: usize,
        count: usize,
    ) -> Result<(), RuntimeError> {
        /* check source and destination for out of bounds accesses */
        // Specification step 12.
        let source_end_index = source_index
            .checked_add(count)
            .ok_or(TrapError::MemoryOrDataAccessOutOfBounds)?;

        let destination_end_index = destination_index
            .checked_add(count)
            .ok_or(TrapError::MemoryOrDataAccessOutOfBounds)?;

        let source_range = source_index..source_end_index;
        let destination_range = destination_index..destination_end_index;

        let source_range_is_within_bounds = self.data.get(source_range.clone()).is_some();
        let destination_range_is_within_bounds = self.data.get(destination_range).is_some();

        if !source_range_is_within_bounds || !destination_range_is_within_bounds {
            return Err(TrapError::MemoryOrDataAccessOutOfBounds.into());
        }

        /* do the potentially-overlapping copy, does nothing if count was zero */
        // Specification step 13-15

        // This cannot panic, because both ranges are within bounds.
        self.data.copy_within(source_range, destination_index);

        Ok(())
    }

    /// Copies a number of bytes from the given byte slice `source_data` into this linear memory. A
    /// total of `count` bytes will be copied, starting at the given source and destination indices.
    ///
    /// # Errors
    ///
    /// - [`TrapError::MemoryOrDataAccessOutOfBounds`]: This operation would have been out of
    ///   bounds. The memory remains unchanged.
    pub fn init(
        &mut self,
        destination_index: usize,
        source_data: &[u8],
        source_index: usize,
        count: usize,
    ) -> Result<(), RuntimeError> {
        /* check source and destination for out of bounds accesses */
        // Specification step 16.
        let source_end_index = source_index
            .checked_add(count)
            .ok_or(TrapError::MemoryOrDataAccessOutOfBounds)?;

        let destination_end_index = destination_index
            .checked_add(count)
            .ok_or(TrapError::MemoryOrDataAccessOutOfBounds)?;

        let source_range = source_index..source_end_index;
        let destination_range = destination_index..destination_end_index;

        let source_bytes = source_data
            .get(source_range)
            .ok_or(TrapError::MemoryOrDataAccessOutOfBounds)?;
        let destination_bytes = self
            .data
            .get_mut(destination_range)
            .ok_or(TrapError::MemoryOrDataAccessOutOfBounds)?;

        /* do the init, also check if there is anything to be done */
        // Specification step 17-27.
        destination_bytes.copy_from_slice(source_bytes);

        Ok(())
    }

    /// Returns the data in this memory as a byte slice. The length of this slice is always a
    /// multiple of `PAGE_SIZE`.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Returns the data in this memory as a mutable byte slice. The length of this slice is always
    /// a multiple of `PAGE_SIZE`.
    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }
}

impl core::fmt::Debug for LinearMemory {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        /// A helper struct for formatting, able to detect and format byte repetitions in a compact
        /// way.
        struct RepetitionDetectingMemoryWriter<'a>(&'a [u8]);
        impl core::fmt::Debug for RepetitionDetectingMemoryWriter<'_> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                /// The number of repetitions required for successive elements to be grouped
                // together.
                const MIN_REPETITIONS_FOR_GROUP: usize = 8;

                // First we create an iterator over all bytes
                let mut bytes = self.0.iter();

                // Then we iterate over all bytes and deduplicate repetitions. This produces an
                // iterator of pairs, consisting of the number of repetitions and the repeated byte
                // itself. `current_group` is captured by the iterator and used as state to track
                // the current group.
                let mut current_group: Option<(usize, u8)> = None;
                let deduplicated_with_count = iter::from_fn(|| {
                    for &byte in bytes.by_ref() {
                        // If the next byte is different than the one being tracked currently...
                        if current_group.is_some() && current_group.unwrap().1 != byte {
                            // ...then end and emit the current group but also start a new group for
                            // the next byte with an initial count of 1.
                            return current_group.replace((1, byte));
                        }
                        // Otherwise increment the current group's counter or start a new group if
                        // this was the first byte.
                        current_group.get_or_insert((0, byte)).0 += 1;
                    }
                    // In the end when there are no more bytes to read, directly emit the last
                    current_group.take()
                });

                // Finally we use `DebugList` to print a list of all groups, while writing out all
                // elements from groups with less than `MIN_REPETITIONS_FOR_GROUP` elements.
                let mut list = f.debug_list();
                deduplicated_with_count.for_each(|(count, value)| {
                    if count < MIN_REPETITIONS_FOR_GROUP {
                        list.entries(iter::repeat_n(value, count));
                    } else {
                        list.entry(&format_args!("#{count} × {value}"));
                    }
                });
                list.finish()
            }
        }

        // Format the linear memory by using Rust's formatter helpers and the previously defined
        // `RepetitionDetectingMemoryWriter`
        f.debug_struct("LinearMemory")
            .field("inner_data", &RepetitionDetectingMemoryWriter(&self.data))
            .finish()
    }
}

#[cfg(test)]
mod test {
    use core::f64;

    use alloc::format;
    use core::mem;

    use crate::{F32, F64};

    use super::*;

    const PAGE_SIZE: NonZeroUsize = NonZeroUsize::new(1 << 8).unwrap();
    const PAGES: usize = 2;

    #[test]
    fn new_constructor() {
        let lin_mem = LinearMemory::new_with_initial_pages::<()>(PAGE_SIZE, 0, None).unwrap();
        assert_eq!(lin_mem.len_pages(), 0);
    }

    #[test]
    fn new_grow() {
        let mut lin_mem = LinearMemory::new_with_initial_pages::<()>(PAGE_SIZE, 0, None).unwrap();
        lin_mem.grow::<()>(1, None).unwrap();
        assert_eq!(lin_mem.len_pages(), 1);
    }

    #[test]
    fn debug_print_simple() {
        let lin_mem = LinearMemory::new_with_initial_pages::<()>(PAGE_SIZE, 1, None).unwrap();
        assert_eq!(lin_mem.len_pages(), 1);

        let expected = format!("LinearMemory {{ inner_data: [#{PAGE_SIZE} × 0] }}");
        let debug_repr = format!("{lin_mem:?}");

        assert_eq!(debug_repr, expected);
    }

    #[test]
    fn debug_print_complex() {
        let page_count = 2;
        let mut lin_mem =
            LinearMemory::new_with_initial_pages::<()>(PAGE_SIZE, page_count, None).unwrap();
        assert_eq!(lin_mem.len_pages(), page_count);

        lin_mem.store(1, 0xffu8).unwrap();
        lin_mem.store(10, 1u8).unwrap();
        lin_mem.store(200, 0xffu8).unwrap();

        let expected = "LinearMemory { inner_data: [0, 255, #8 × 0, 1, #189 × 0, 255, #311 × 0] }";
        let debug_repr = format!("{lin_mem:?}");

        assert_eq!(debug_repr, expected);
    }

    #[test]
    fn debug_print_empty() {
        let lin_mem = LinearMemory::new_with_initial_pages::<()>(PAGE_SIZE, 0, None).unwrap();
        assert_eq!(lin_mem.len_pages(), 0);

        let expected = "LinearMemory { inner_data: [] }";
        let debug_repr = format!("{lin_mem:?}");

        assert_eq!(debug_repr, expected);
    }

    #[test]
    fn roundtrip_normal_range_i8_neg127() {
        let x: i8 = -127;
        let highest_legal_offset = PAGE_SIZE.get() - mem::size_of::<i8>();
        for offset in 0..highest_legal_offset {
            let mut lin_mem =
                LinearMemory::new_with_initial_pages::<()>(PAGE_SIZE, PAGES, None).unwrap();

            lin_mem.store(offset, x).unwrap();

            assert_eq!(
                lin_mem
                    .load::<{ core::mem::size_of::<i8>() }, i8>(offset)
                    .unwrap(),
                x,
                "load store roundtrip for {x:?} failed!"
            );
        }
    }

    #[test]
    fn roundtrip_normal_range_f32_13() {
        let x = F32(13.0);
        let highest_legal_offset = PAGE_SIZE.get() - mem::size_of::<F32>();
        for offset in 0..highest_legal_offset {
            let mut lin_mem =
                LinearMemory::new_with_initial_pages::<()>(PAGE_SIZE, PAGES, None).unwrap();

            lin_mem.store(offset, x).unwrap();

            assert_eq!(
                lin_mem
                    .load::<{ core::mem::size_of::<F32>() }, F32>(offset)
                    .unwrap(),
                x,
                "load store roundtrip for {x:?} failed!"
            );
        }
    }

    #[test]
    fn roundtrip_normal_range_f64_min() {
        let x = F64(f64::MIN);
        let highest_legal_offset = PAGE_SIZE.get() - mem::size_of::<F64>();
        for offset in 0..highest_legal_offset {
            let mut lin_mem =
                LinearMemory::new_with_initial_pages::<()>(PAGE_SIZE, PAGES, None).unwrap();

            lin_mem.store(offset, x).unwrap();

            assert_eq!(
                lin_mem
                    .load::<{ core::mem::size_of::<F64>() }, F64>(offset)
                    .unwrap(),
                x,
                "load store roundtrip for {x:?} failed!"
            );
        }
    }

    #[test]
    fn roundtrip_normal_range_f64_nan() {
        let x = F64(f64::NAN);
        let highest_legal_offset = PAGE_SIZE.get() - mem::size_of::<f64>();
        for offset in 0..highest_legal_offset {
            let mut lin_mem =
                LinearMemory::new_with_initial_pages::<()>(PAGE_SIZE, PAGES, None).unwrap();

            lin_mem.store(offset, x).unwrap();

            assert!(
                lin_mem
                    .load::<{ core::mem::size_of::<F64>() }, F64>(offset)
                    .unwrap()
                    .is_nan(),
                "load store roundtrip for {x:?} failed!"
            );
        }
    }

    #[test]
    #[should_panic(
        expected = "called `Result::unwrap()` on an `Err` value: MemoryOrDataAccessOutOfBounds"
    )]
    fn store_out_of_range_u128_max() {
        let x: u128 = u128::MAX;
        let pages = 1;
        let lowest_illegal_offset = PAGE_SIZE.get() - mem::size_of::<u128>() + 1;
        let mut lin_mem =
            LinearMemory::new_with_initial_pages::<()>(PAGE_SIZE, pages, None).unwrap();

        lin_mem.store(lowest_illegal_offset, x).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "called `Result::unwrap()` on an `Err` value: MemoryOrDataAccessOutOfBounds"
    )]
    fn store_empty_lineaer_memory_u8() {
        let x: u8 = u8::MAX;
        let pages = 0;
        let lowest_illegal_offset = PAGE_SIZE.get() - mem::size_of::<u8>() + 1;
        let mut lin_mem =
            LinearMemory::new_with_initial_pages::<()>(PAGE_SIZE, pages, None).unwrap();

        lin_mem.store(lowest_illegal_offset, x).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "called `Result::unwrap()` on an `Err` value: MemoryOrDataAccessOutOfBounds"
    )]
    fn load_out_of_range_u128_max() {
        let pages = 1;
        let lowest_illegal_offset = PAGE_SIZE.get() - mem::size_of::<u128>() + 1;
        let lin_mem = LinearMemory::new_with_initial_pages::<()>(PAGE_SIZE, pages, None).unwrap();

        let _x: u128 = lin_mem.load(lowest_illegal_offset).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "called `Result::unwrap()` on an `Err` value: MemoryOrDataAccessOutOfBounds"
    )]
    fn load_empty_lineaer_memory_u8() {
        let pages = 0;
        let lowest_illegal_offset = PAGE_SIZE.get() - mem::size_of::<u8>() + 1;
        let lin_mem = LinearMemory::new_with_initial_pages::<()>(PAGE_SIZE, pages, None).unwrap();

        let _x: u8 = lin_mem.load(lowest_illegal_offset).unwrap();
    }

    #[test]
    #[should_panic]
    fn copy_out_of_bounds() {
        let mut lin_mem_0 = LinearMemory::new_with_initial_pages::<()>(PAGE_SIZE, 2, None).unwrap();
        lin_mem_0
            .copy_within(0, PAGE_SIZE.get(), PAGE_SIZE.get() + 1)
            .unwrap();
    }
}
