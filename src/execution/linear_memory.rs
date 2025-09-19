use core::{cell::UnsafeCell, iter, ptr};

use alloc::vec::Vec;

use crate::{
    core::indices::MemIdx,
    execution::little_endian::LittleEndianBytes,
    rw_spinlock::{ReadLockGuard, RwSpinLock},
    RuntimeError, TrapError,
};

/// Implementation of the linear memory suitable for concurrent access
///
/// Implements the base for the instructions described in
/// <https://webassembly.github.io/spec/core/exec/instructions.html#memory-instructions>.
///
/// This linear memory implementation internally relies on a `Vec<UnsafeCell<u8>>`. Thus, the atomic
/// unit of information for it is a byte (`u8`). All access to the linear memory internally occurs
/// through pointers, avoiding the creation of shared and mut refs to the internal data completely.
/// This avoids undefined behavior, except for the race-condition inherent to concurrent writes.
/// Because of this, the [`LinearMemory::store`] function does not require `&mut self` -- `&self`
/// suffices.
///
/// # Notes on overflowing
///
/// All operations that rely on accessing `n` bytes starting at `index` in the linear memory have to
/// perform bounds checking. Thus they always have to ensure that `n + index < linear_memory.len()`
/// holds true (e.g. `n + index - 1` must be a valid index into `linear_memory`). However,
/// writing that check as is bears the danger of an overflow, assuming that `n`, `index` and
/// `linear_memory.len()` are the same given integer type, `n + index` can overflow, resulting in
/// the check passing despite the access being out of bounds!
///
/// To avoid this, the bounds checks are carefully ordered to avoid any overflows:
///
/// - First we check, that `n <= linear_memory.len()` holds true, ensuring that the amount of bytes
///   to be accessed is indeed smaller than or equal to the linear memory's size. If this does not
///   hold true, continuation of the operation will yield out of bounds access in any case.
/// - Then, as a second check, we verify that `index <= linear_memory.len() - n`. This way we
///   avoid the overflow, as there is no addition. The subtraction in the left hand can not
///   underflow, due to the previous check (which asserts that `n` is smaller than or equal to
///   `linear_memory.len()`).
///
/// Combined in the given order, these two checks enable bounds checking without risking any
/// overflow or underflow, provided that `n`, `index` and `linear_memory.len()` are of the same
/// integer type.
///
/// # Notes on locking
///
/// The internal data vector of the [`LinearMemory`] is wrapped in a [`RwSpinLock`]. Despite the
/// name, writes to the linear memory do not require an acquisition of a write lock. Writes are
/// implemented through a shared ref to the internal vector, with an `UnsafeCell` to achieve
/// interior mutability.
///
/// However, linear memory can grow. As the linear memory is implemented via a [`Vec`], a grow can
/// result in the vector's internal data buffer to be copied over to a bigger, fresh allocation.
/// The old buffer is then freed. Combined with concurrent mutable access, this can cause
/// use-after-free. To avoid this, a grow operation of the linear memory acquires a write lock,
/// blocking all read/write to the linear memory inbetween.
///
/// # Unsafe Note
///
/// Raw pointer access it required, because concurent mutation of the linear memory might happen
/// (consider the threading proposal for WASM, where mutliple WASM threads access the same linear
/// memory at the same time). The inherent race condition results in UB w/r/t the state of the `u8`s
/// in the inner data. However, this is tolerable, e.g. avoiding race conditions on the state of the
/// linear memory can not be the task of the interpreter, but has to be fulfilled by the interpreted
/// bytecode itself.
///
/// To gain some confidence in the correctness of the unsafe code in this module, run `miri`:
///
/// ```bash
/// cargo miri test --test memory # quick
/// cargo miri test # thorough
/// ```
// TODO if a memmap like operation is available, the linear memory implementation can be optimized brutally. Out-of-bound access can be mapped to userspace handled page-faults, e.g. the MMU takes over that responsibility of catching out of bounds. Grow can happen without copying of data, by mapping new pages consecutively after the current final page of the linear memory.
pub struct LinearMemory<const PAGE_SIZE: usize = { crate::Limits::MEM_PAGE_SIZE as usize }> {
    inner_data: RwSpinLock<Vec<UnsafeCell<u8>>>,
}

/// Type to express the page count
pub type PageCountTy = u16;

impl<const PAGE_SIZE: usize> LinearMemory<PAGE_SIZE> {
    /// Size of a page in the linear memory, measured in bytes
    ///
    /// The WASM specification demands a page size of 64 KiB, that is `65536` bytes:
    /// <https://webassembly.github.io/spec/core/exec/runtime.html?highlight=page#memory-instances>
    const PAGE_SIZE: usize = PAGE_SIZE;

    /// Create a new, empty [`LinearMemory`]
    pub fn new() -> Self {
        Self {
            inner_data: RwSpinLock::new(Vec::new()),
        }
    }

    /// Create a new, empty [`LinearMemory`]
    pub fn new_with_initial_pages(pages: PageCountTy) -> Self {
        let size_bytes = Self::PAGE_SIZE * pages as usize;
        let mut data = Vec::with_capacity(size_bytes);
        data.resize_with(size_bytes, || UnsafeCell::new(0));

        Self {
            inner_data: RwSpinLock::new(data),
        }
    }

    /// Grow the [`LinearMemory`] by a number of pages
    pub fn grow(&self, pages_to_add: PageCountTy) {
        let mut lock_guard = self.inner_data.write();
        let prior_length_bytes = lock_guard.len();
        let new_length_bytes = prior_length_bytes + Self::PAGE_SIZE * pages_to_add as usize;
        lock_guard.resize_with(new_length_bytes, || UnsafeCell::new(0));
    }

    /// Get the number of pages currently allocated to this [`LinearMemory`]
    pub fn pages(&self) -> PageCountTy {
        PageCountTy::try_from(self.inner_data.read().len() / PAGE_SIZE).unwrap()
    }

    /// Get the length in bytes currently allocated to this [`LinearMemory`]
    // TODO remove this op
    pub fn len(&self) -> usize {
        self.inner_data.read().len()
    }

    /// At a given index, store a datum in the [`LinearMemory`]
    pub fn store<const N: usize, T: LittleEndianBytes<N>>(
        &self,
        index: MemIdx,
        value: T,
    ) -> Result<(), RuntimeError> {
        let lock_guard = self.inner_data.read();

        /* check destination for out of bounds access */
        // A value must fit into the linear memory
        if N > lock_guard.len() {
            error!("value does not fit into linear memory");
            return Err(TrapError::MemoryOrDataAccessOutOfBounds.into());
        }

        // The following statement must be true
        // `index + N <= lock_guard.len()`
        // This check verifies it, while avoiding the possible overflow. The subtraction can not
        // underflow because of the previous check.

        if index > lock_guard.len() - N {
            error!("value write would extend beyond the end of the linear memory");
            return Err(TrapError::MemoryOrDataAccessOutOfBounds.into());
        }

        let bytes = value.to_le_bytes();

        /* gather pointers */
        let src_ptr = bytes.as_ptr();
        let dst_ptr = UnsafeCell::raw_get(lock_guard.as_ptr());

        /* write `value` to this `LinearMemory` */

        // SAFETY:
        // - nonoverlapping is guaranteed, because `src_ptr` is a pointer to a stack allocated
        //   array, while `dst_ptr` points to a heap allocated `Vec`
        // - the first if statement in this function guarantees that a `T` can fit into
        //   `LinearMemory` behind the `dst_ptr`
        // - the second if statement in this function guarantees that even with the offset
        //   `index`, writing all of `src_ptr`'s bytes does not extend beyond the `dst_ptr`'s last
        //   `UnsafeCell<u8>`
        // - the use of `UnsafeCell` avoids any `&` or `&mut` to ever be created on any of the `u8`s
        //   contained in the `UnsafeCell`s, so no UB is created through the existence of unsound
        //   references
        unsafe { ptr::copy_nonoverlapping(src_ptr, dst_ptr.add(index), bytes.len()) };

        Ok(())
    }

    /// From a given index, load a datum in the [`LinearMemory`]
    pub fn load<const N: usize, T: LittleEndianBytes<N>>(
        &self,
        index: MemIdx,
    ) -> Result<T, RuntimeError> {
        let lock_guard = self.inner_data.read();

        /* check source for out of bounds access */
        // A value must fit into the linear memory
        if N > lock_guard.len() {
            error!("value does not fit into linear memory");
            return Err(TrapError::MemoryOrDataAccessOutOfBounds.into());
        }

        // The following statement must be true
        // `index + N <= lock_guard.len()`
        // This check verifies it, while avoiding the possible overflow. The subtraction can not
        // underflow because of the previous assert.

        if index > lock_guard.len() - N {
            error!("value read would extend beyond the end of the linear_memory");
            return Err(TrapError::MemoryOrDataAccessOutOfBounds.into());
        }

        let mut bytes = [0; N];

        /* gather pointers */
        let src_ptr = UnsafeCell::raw_get(lock_guard.as_ptr());
        let dst_ptr = bytes.as_mut_ptr();

        /* read `value` from this `LinearMemory` */
        // SAFETY:
        // - nonoverlapping is guaranteed, because `dst_ptr` is a pointer to a stack allocated
        //   array, while the source is heap allocated Vec
        // - the first if statement in this function guarantees that a `T` can fit into the linear
        //   memory behind the `src_ptr`
        // - the second if statement in this function guarantees that even with the offset `index`,
        //   reading all of `T`s bytes does not extend beyond the `src_ptrs`'s last `UnsafeCell<u8>`
        // - the use of `UnsafeCell` avoids any `&` or `&mut` to ever be created on any of the `u8`s
        //   contained in the `UnsafeCell`s, so no UB is created through the existence of unsound
        //   references
        unsafe { ptr::copy_nonoverlapping(src_ptr.add(index), dst_ptr, bytes.len()) };

        Ok(T::from_le_bytes(bytes))
    }

    /// Implementation of the behavior described in
    /// <https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-memory-mathsf-memory-fill>.
    /// Note, that the WASM spec defines the behavior by recursion, while our implementation uses
    /// the memset like [`core::ptr::write_bytes`].
    ///
    /// <https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-memory-mathsf-memory-fill>
    pub fn fill(&self, index: MemIdx, data_byte: u8, count: MemIdx) -> Result<(), RuntimeError> {
        let lock_guard = self.inner_data.read();

        /* check destination for out of bounds access */
        // Specification step 12.
        if count > lock_guard.len() {
            error!("fill count is bigger than the linear memory");
            return Err(TrapError::MemoryOrDataAccessOutOfBounds.into());
        }

        // Specification step 12.
        if index > lock_guard.len() - count {
            error!("fill extends beyond the linear memory's end");
            return Err(TrapError::MemoryOrDataAccessOutOfBounds.into());
        }

        /* check if there is anything to be done */
        // Specification step 13.
        if count == 0 {
            return Ok(());
        }

        /* gather pointer */
        let dst_ptr = UnsafeCell::raw_get(lock_guard.as_ptr());

        /* write the `data_byte` to this `LinearMemory` */

        // SAFETY:
        // - the first if statement of this function guarantees that count fits into this
        //   `LinearMemory`
        // - the second if statement of this function guarantees that even with the offset `index`,
        //   `count` many bytes can be written to this `LinearMemory` without extending beyond its
        //   last `UnsafeCell<u8>`
        // - the use of `UnsafeCell` avoids any `&` or `&mut` to ever be created on any of the `u8`s
        //   contained in the `UnsafeCell`s, so no UB is created through the existence of unsound
        //   references

        // Specification step 14-21.
        unsafe { dst_ptr.add(index).write_bytes(data_byte, count) };

        Ok(())
    }

    /// Copy `count` bytes from one region in the linear memory to another region in the same or a
    /// different linear memory
    ///
    /// - Both regions may overlap
    /// - Copies the `count` bytes starting from `source_index`, overwriting the `count` bytes
    ///   starting from `destination_index`
    ///
    /// <https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-memory-mathsf-memory-copy>
    pub fn copy(
        &self,
        destination_index: MemIdx,
        source_mem: &Self,
        source_index: MemIdx,
        count: MemIdx,
    ) -> Result<(), RuntimeError> {
        // self is the destination
        let lock_guard_self = self.inner_data.read();

        // other is the source
        let lock_guard_other = source_mem.inner_data.read();

        /* check source for out of bounds access */
        // Specification step 12.
        if count > lock_guard_other.len() {
            error!("copy count is bigger than the source linear memory");
            return Err(TrapError::MemoryOrDataAccessOutOfBounds.into());
        }

        // Specification step 12.
        if source_index > lock_guard_other.len() - count {
            error!("copy source extends beyond the linear memory's end");
            return Err(TrapError::MemoryOrDataAccessOutOfBounds.into());
        }

        /* check destination for out of bounds access */
        // Specification step 12.
        if count > lock_guard_self.len() {
            error!("copy count is bigger than the destination linear memory");
            return Err(TrapError::MemoryOrDataAccessOutOfBounds.into());
        }

        // Specification step 12.
        if destination_index > lock_guard_self.len() - count {
            error!("copy destination extends beyond the linear memory's end");
            return Err(TrapError::MemoryOrDataAccessOutOfBounds.into());
        }

        /* check if there is anything to be done */
        // Specification step 13.
        if count == 0 {
            return Ok(());
        }

        /* gather pointers */
        let src_ptr = UnsafeCell::raw_get(lock_guard_other.as_ptr());
        let dst_ptr = UnsafeCell::raw_get(lock_guard_self.as_ptr());

        /* write from `source_mem` to `self` */

        // SAFETY:
        // - the first two if statements above guarantee that starting from `source_index`,
        //   there are at least `count` further `UnsafeCell<u8>`s in the other `LinearMemory`
        // - the third and fourth if statement above guarantee that starting from
        //   `destination_index`, there are at least `count` further `UnsafeCell<u8>`s in this
        //   `LinearMemory`
        // - the use of `UnsafeCell` avoids any `&` or `&mut` to ever be created on any of the `u8`s
        //   contained in the `UnsafeCell`s, so no UB is created through the existence of unsound
        //   references
        // - as per the other statements above, both `*_ptr` are valid, and have at least `count`
        //   further values after them in their respective `LinearMemory`s
        // - the use of `UnsafeCell` avoids any `&` or `&mut` to ever be created on any of the `u8`s
        //   contained in the `UnsafeCell`s, so no UB is created through the existence of unsound
        //   references

        // Specification step 14-15.
        // TODO investigate if it is worth to use a conditional `copy_from_nonoverlapping`
        // if the non-overlapping can be confirmed (and the count is bigger than a certain
        // threshold).
        unsafe {
            ptr::copy(
                src_ptr.add(source_index),
                dst_ptr.add(destination_index),
                count,
            )
        }

        Ok(())
    }

    // Rationale behind having `source_index` and `count` when the callsite could also just create a
    // subslice for `source_data`? Have all the index error checks in one place.
    //
    // <https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-memory-mathsf-memory-init-x>
    pub fn init(
        &self,
        destination_index: MemIdx,
        source_data: &[u8],
        source_index: MemIdx,
        count: MemIdx,
    ) -> Result<(), RuntimeError> {
        // self is the destination
        let lock_guard_self = self.inner_data.read();
        let data_len = source_data.len();

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
        if count > lock_guard_self.len() {
            error!("init count is bigger than the linear memory");
            return Err(TrapError::MemoryOrDataAccessOutOfBounds.into());
        }

        // Specification step 16.
        if destination_index > lock_guard_self.len() - count {
            error!("init extends beyond the linear memory's end");
            return Err(TrapError::MemoryOrDataAccessOutOfBounds.into());
        }

        /* check if there is anything to be done */
        // Specification step 17.
        if count == 0 {
            return Ok(());
        }

        /* copy the data to this `LinearMemory` */

        // Specification step 18-27.
        for i in 0..count {
            // SAFETY: this is sound, as the two if statements above guarantee that starting from
            // `source_index`, there are at least `count` further `u8`s in `source_data`
            let src_ptr = unsafe { source_data.get_unchecked(source_index + i) };

            // SAFETY: this is sound, as the two if statements above guarantee that starting from
            // `destination_index`, there are at least `count` further `UnsafeCell<u8>`s in this
            // `LinearMemory`
            let dst_ptr = unsafe { lock_guard_self.get_unchecked(destination_index + i) }.get();

            // SAFETY:
            // - as per the other SAFETY statements in this function, both `*_ptr` are valid, and
            //   have at least `count` further values after them in them respectively
            // - the use of `UnsafeCell` avoids any `&` or `&mut` to ever be created on any of the
            //   `u8`s contained in the `UnsafeCell`s, so no UB is created through the existence of
            //   unsound references
            unsafe {
                ptr::copy(src_ptr, dst_ptr, 1);
            }
        }

        Ok(())
    }
}

impl<const PAGE_SIZE: usize> core::fmt::Debug for LinearMemory<PAGE_SIZE> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        /// A helper struct for formatting a [`Vec<UnsafeCell<u8>>`] which is guarded by a [`ReadLockGuard`].
        /// This formatter is able to detect and format byte repetitions in a compact way.
        struct RepetitionDetectingMemoryWriter<'a>(ReadLockGuard<'a, Vec<UnsafeCell<u8>>>);
        impl core::fmt::Debug for RepetitionDetectingMemoryWriter<'_> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                /// The number of repetitions required for successive elements to be grouped
                // together.
                const MIN_REPETITIONS_FOR_GROUP: usize = 8;

                // First we create an iterator over all bytes
                let mut bytes = self.0.iter().map(|x| {
                    // SAFETY: The [`ReadLockGuard`] stored in `self` prevents a resize/realloc of
                    // its data, so access to the value inside each [`UnsafeCell`] is safe.
                    unsafe { *x.get() }
                });

                // Then we iterate over all bytes and deduplicate repetitions. This produces an
                // iterator of pairs, consisting of the number of repetitions and the repeated byte
                // itself. `current_group` is captured by the iterator and used as state to track
                // the current group.
                let mut current_group: Option<(usize, u8)> = None;
                let deduplicated_with_count = iter::from_fn(|| {
                    for byte in bytes.by_ref() {
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
                        list.entries(iter::repeat(value).take(count));
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
            .field(
                "inner_data",
                &RepetitionDetectingMemoryWriter(self.inner_data.read()),
            )
            .finish()
    }
}

impl<const PAGE_SIZE: usize> Default for LinearMemory<PAGE_SIZE> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use alloc::format;
    use core::mem;

    use super::*;

    const PAGE_SIZE: usize = 1 << 8;
    const PAGES: PageCountTy = 2;

    #[test]
    fn new_constructor() {
        let lin_mem = LinearMemory::<PAGE_SIZE>::new();
        assert_eq!(lin_mem.pages(), 0);
    }

    #[test]
    fn new_grow() {
        let lin_mem = LinearMemory::<PAGE_SIZE>::new();
        lin_mem.grow(1);
        assert_eq!(lin_mem.pages(), 1);
    }

    #[test]
    fn debug_print_simple() {
        let lin_mem = LinearMemory::<PAGE_SIZE>::new_with_initial_pages(1);
        assert_eq!(lin_mem.pages(), 1);

        let expected = format!("LinearMemory {{ inner_data: [#{PAGE_SIZE} × 0] }}");
        let debug_repr = format!("{lin_mem:?}");

        assert_eq!(debug_repr, expected);
    }

    #[test]
    fn debug_print_complex() {
        let page_count = 2;
        let lin_mem = LinearMemory::<PAGE_SIZE>::new_with_initial_pages(page_count);
        assert_eq!(lin_mem.pages(), page_count);

        lin_mem.store(1, 0xffu8).unwrap();
        lin_mem.store(10, 1u8).unwrap();
        lin_mem.store(200, 0xffu8).unwrap();

        let expected = "LinearMemory { inner_data: [0, 255, #8 × 0, 1, #189 × 0, 255, #311 × 0] }";
        let debug_repr = format!("{lin_mem:?}");

        assert_eq!(debug_repr, expected);
    }

    #[test]
    fn debug_print_empty() {
        let lin_mem = LinearMemory::<PAGE_SIZE>::new_with_initial_pages(0);
        assert_eq!(lin_mem.pages(), 0);

        let expected = "LinearMemory { inner_data: [] }";
        let debug_repr = format!("{lin_mem:?}");

        assert_eq!(debug_repr, expected);
    }

    #[test]
    fn roundtrip_normal_range_i8_neg127() {
        let x: i8 = -127;
        let highest_legal_offset = PAGE_SIZE - mem::size_of::<i8>();
        for offset in 0..MemIdx::try_from(highest_legal_offset).unwrap() {
            let lin_mem = LinearMemory::<PAGE_SIZE>::new_with_initial_pages(PAGES);

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
        let x: f32 = 13.0;
        let highest_legal_offset = PAGE_SIZE - mem::size_of::<f32>();
        for offset in 0..MemIdx::try_from(highest_legal_offset).unwrap() {
            let lin_mem = LinearMemory::<PAGE_SIZE>::new_with_initial_pages(PAGES);

            lin_mem.store(offset, x).unwrap();

            assert_eq!(
                lin_mem
                    .load::<{ core::mem::size_of::<f32>() }, f32>(offset)
                    .unwrap(),
                x,
                "load store roundtrip for {x:?} failed!"
            );
        }
    }

    #[test]
    fn roundtrip_normal_range_f64_min() {
        let x: f64 = f64::MIN;
        let highest_legal_offset = PAGE_SIZE - mem::size_of::<f64>();
        for offset in 0..MemIdx::try_from(highest_legal_offset).unwrap() {
            let lin_mem = LinearMemory::<PAGE_SIZE>::new_with_initial_pages(PAGES);

            lin_mem.store(offset, x).unwrap();

            assert_eq!(
                lin_mem
                    .load::<{ core::mem::size_of::<f64>() }, f64>(offset)
                    .unwrap(),
                x,
                "load store roundtrip for {x:?} failed!"
            );
        }
    }

    #[test]
    fn roundtrip_normal_range_f64_nan() {
        let x: f64 = f64::NAN;
        let highest_legal_offset = PAGE_SIZE - mem::size_of::<f64>();
        for offset in 0..MemIdx::try_from(highest_legal_offset).unwrap() {
            let lin_mem = LinearMemory::<PAGE_SIZE>::new_with_initial_pages(PAGES);

            lin_mem.store(offset, x).unwrap();

            assert!(
                lin_mem
                    .load::<{ core::mem::size_of::<f64>() }, f64>(offset)
                    .unwrap()
                    .is_nan(),
                "load store roundtrip for {x:?} failed!"
            );
        }
    }

    #[test]
    #[should_panic(
        expected = "called `Result::unwrap()` on an `Err` value: Trap(MemoryOrDataAccessOutOfBounds)"
    )]
    fn store_out_of_range_u128_max() {
        let x: u128 = u128::MAX;
        let pages = 1;
        let lowest_illegal_offset = PAGE_SIZE - mem::size_of::<u128>() + 1;
        let lowest_illegal_offset = MemIdx::try_from(lowest_illegal_offset).unwrap();
        let lin_mem = LinearMemory::<PAGE_SIZE>::new_with_initial_pages(pages);

        lin_mem.store(lowest_illegal_offset, x).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "called `Result::unwrap()` on an `Err` value: Trap(MemoryOrDataAccessOutOfBounds)"
    )]
    fn store_empty_lineaer_memory_u8() {
        let x: u8 = u8::MAX;
        let pages = 0;
        let lowest_illegal_offset = PAGE_SIZE - mem::size_of::<u8>() + 1;
        let lowest_illegal_offset = MemIdx::try_from(lowest_illegal_offset).unwrap();
        let lin_mem = LinearMemory::<PAGE_SIZE>::new_with_initial_pages(pages);

        lin_mem.store(lowest_illegal_offset, x).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "called `Result::unwrap()` on an `Err` value: Trap(MemoryOrDataAccessOutOfBounds)"
    )]
    fn load_out_of_range_u128_max() {
        let pages = 1;
        let lowest_illegal_offset = PAGE_SIZE - mem::size_of::<u128>() + 1;
        let lowest_illegal_offset = MemIdx::try_from(lowest_illegal_offset).unwrap();
        let lin_mem = LinearMemory::<PAGE_SIZE>::new_with_initial_pages(pages);

        let _x: u128 = lin_mem.load(lowest_illegal_offset).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "called `Result::unwrap()` on an `Err` value: Trap(MemoryOrDataAccessOutOfBounds)"
    )]
    fn load_empty_lineaer_memory_u8() {
        let pages = 0;
        let lowest_illegal_offset = PAGE_SIZE - mem::size_of::<u8>() + 1;
        let lowest_illegal_offset = MemIdx::try_from(lowest_illegal_offset).unwrap();
        let lin_mem = LinearMemory::<PAGE_SIZE>::new_with_initial_pages(pages);

        let _x: u8 = lin_mem.load(lowest_illegal_offset).unwrap();
    }

    #[test]
    #[should_panic]
    fn copy_out_of_bounds() {
        let lin_mem_0 = LinearMemory::<PAGE_SIZE>::new_with_initial_pages(2);
        let lin_mem_1 = LinearMemory::<PAGE_SIZE>::new_with_initial_pages(1);
        lin_mem_0.copy(0, &lin_mem_1, 0, PAGE_SIZE + 1).unwrap();
    }
}
