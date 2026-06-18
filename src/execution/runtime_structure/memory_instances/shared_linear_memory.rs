use core::{
    iter,
    mem::MaybeUninit,
    num::NonZeroUsize,
    sync::{self, atomic::AtomicU8},
};

use alloc::vec::Vec;

use crate::{
    execution::{
        numerics::representations::LittleEndianBytes,
        runtime_structure::memory_instances::DEFAULT_PAGE_SIZE,
    },
    rw_spinlock::{ReadLockGuard, RwSpinLock, WriteLockGuard},
    Config, RuntimeError, TrapError,
};

/// Implementation of a shared linear memory, as defined in the threads proposal[^threads-proposal].
///
/// Unlike unshared linear memories which store their data as a vector of bytes, a shared linear
/// memory's backing data structure is not specified.
///
/// Instead, a number of so-called _actions_, which are basically the side-effects of certain
/// instructions, are defined. The following three types of actions are implemented as of now:
///
/// - `rd`: Reads the length or data of the memory. ([`Self::rd_len`], [`Self::rd_data`])
/// - `wr`: Writes data into the memory. ([`Self::wr_data`])
/// - `rmw`: Reads something from, modifies it and writes it back into the memory. This can be
///   either data or the length of the memory. ([Self:TODO], ...)
///
/// These actions act as a base for implementing the execution of memory instructions as described
/// in the threads proposal[^threads-proposal].
///
/// [^threads-proposal]: [Threads Proposal Repository](https://github.com/WebAssembly/threads).
///
/// # Implementation & Locking
///
/// This memory internally relies on a [`Vec<AtomicU8>`]. Thus, the atomic unit of information for
/// it is a byte (`u8`). All byte-wise accesses to the linear memory internally occur through
/// [`AtomicU8::load`] and [`AtomicU8::store`], avoiding the need for an exclusive for these
/// operations.
///
/// However, there a two reasons for why a lock must guard the inner vector of atomic bytes:
///
/// 1. Implementing atomic stores for multi-byte values requires a global write lock. Rust's memory
///    model considers partially overlapping atomic operations involving a write to be undefined
///    behavior. It is also impossible to predict whether an atomic multi-byte store operation might
///    overlap with another operation at runtime, thereby necessitating a lock to gain temporary
///    exclusive access.
/// 2. Linear memory can grow and while shared memories must always have an upper limit, we choose
///    not to pre-allocate the entire memory. Instead we allow growing the inner [`Vec`] allocation,
///    which requires temporary exclusive access to move all data from the old to the new fresh
///    allocation. The old allocation is then freed afterwards.
///
/// TODO: Does it pay of to have more fine-granular locking for multi-byte stores than a single
///       global write lock? Would this cause issues with predictable execution times?
///
/// # Unsafe Note
///
/// As the manual index checking assures all indices to be valid, there is no need to re-check.
/// Therefore [`slice::get_unchecked`] is used access the internal [`AtomicU8`] in the vector
/// backing a [`SharedLinearMemory`], implicating the use of `unsafe`.
///
/// To gain some confidence in the correctness of the unsafe code in this module, run `miri`:
///
/// ```bash
/// cargo miri test --test memory # quick
/// cargo miri test # thorough
/// ```
///
// TODO: Allow rd/wr actions to take values directly instead of byte slices
// TODO: Make user-defined limits (T::MAX_NUMBER_OF_MEMORY_PAGES) use the max_pages field, when we
//       implement dynamically checked custom memory limits.
pub struct SharedLinearMemory {
    /// This vector's length must never be 2^32 or larger.
    inner_data: RwSpinLock<Vec<AtomicU8>>,
    page_size: NonZeroUsize,
    /// We have to store the maximum number of pages in this memory so that we can perform a bounds
    /// when whenever the user calls [`SharedLinearMemory::grow`].
    max_pages: usize,
}

/// A memory ordering used for atomic accesses
///
/// See: [WebAssembly Specification 2.0 (with Threads proposal) - 4.2.17 Events - ord](https://webassembly.github.io/threads/core/exec/runtime.html#events)
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Ordering {
    Unord,
    SeqCst,
    Init,
}

impl Ordering {
    fn into_atomic_ordering(self) -> sync::atomic::Ordering {
        match self {
            Ordering::Unord => sync::atomic::Ordering::Relaxed,
            Ordering::SeqCst => sync::atomic::Ordering::SeqCst,
            Ordering::Init => todo!("handle init ordering"),
        }
    }
}

impl SharedLinearMemory {
    /// Create a new and empty shared linear memory. A maximum for the number of pages allocatable
    /// by this memory must be set.
    pub fn new(max_pages: usize) -> Self {
        Self::new_with_page_size(max_pages, DEFAULT_PAGE_SIZE)
    }

    // This is pub(crate) because we cannot expose custom page sizes to users yet.
    // TODO For the custom page size proposal, rename this to `new`.
    pub(crate) fn new_with_page_size(max_pages: usize, page_size: NonZeroUsize) -> Self {
        Self {
            inner_data: RwSpinLock::new(Vec::new()),
            // For now do not expose the option to set custom page sizes, as the SharedLinearMemory
            // is exposed to users.
            page_size,
            max_pages,
        }
    }

    // This is pub(crate) because we cannot expose custom page sizes to users yet.
    // TODO For the custom page size proposal, rename this to `new_with_initial_pages`.
    pub(crate) fn new_with_initial_pages_and_page_size<T: Config>(
        pages: usize,
        max_pages: usize,
        page_size: NonZeroUsize,
    ) -> Result<Self, RuntimeError> {
        let size_bytes = page_size
            .get()
            .checked_mul(pages)
            .ok_or(RuntimeError::MemoryOverflowed)?;

        if let Some(max_pages) = T::MAX_NUMBER_OF_MEMORY_PAGES {
            if pages > usize::from(max_pages.get()) {
                // TODO Should we return RuntimeError::MemoryExceededLimit instead? Do this after we
                // have proper unverified/verified versions of Limits/MemType (see issue #406).
                return Err(RuntimeError::MemoryOverflowed);
            }
        }

        let mut data = Vec::with_capacity(size_bytes);
        data.resize_with(size_bytes, || AtomicU8::new(0));

        Ok(Self {
            inner_data: RwSpinLock::new(data),
            page_size,
            max_pages,
        })
    }

    /// Create a new shared linear memory with a certain number of zero-initialized pages.
    ///
    /// A maximum for the number of pages allocatable by this memory must also be set.
    pub fn new_with_initial_pages<T: Config>(
        pages: usize,
        max_pages: usize,
    ) -> Result<Self, RuntimeError> {
        Self::new_with_initial_pages_and_page_size::<T>(pages, max_pages, DEFAULT_PAGE_SIZE)
    }

    /// Grow the [`SharedLinearMemory`] by a number of pages, and fills the new bytes with zeros.
    ///
    /// The entire grow operation is atomic and visible to other threads as if it was
    /// sequentially-consistent. Internally it works by temporarily acquiring exclusive access to
    /// the memory's data storage.
    pub fn grow<T: Config>(&self, pages_to_add: usize) -> Result<usize, RuntimeError> {
        let mut lock_guard = self.inner_data.write();

        let prior_length_bytes = lock_guard.len();
        let len_pages = prior_length_bytes / self.page_size;
        if len_pages
            .checked_add(pages_to_add)
            .is_none_or(|new_pages| new_pages > self.max_pages)
        {
            return Err(RuntimeError::MemoryGrowExceededLimit);
        }

        let num_bytes_to_append = self
            .page_size
            .get()
            .checked_mul(pages_to_add)
            .ok_or(RuntimeError::MemoryOverflowed)?;
        let new_length_bytes = prior_length_bytes
            .checked_add(num_bytes_to_append)
            .ok_or(RuntimeError::MemoryOverflowed)?;

        if let Some(max_pages) = T::MAX_NUMBER_OF_MEMORY_PAGES {
            if usize::from(max_pages.get())
                .checked_mul(self.page_size.get())
                .is_some_and(|max_len| new_length_bytes > max_len)
            {
                return Err(RuntimeError::MemoryOverflowed);
            }
        }

        lock_guard.resize_with(new_length_bytes, || AtomicU8::new(0));

        Ok(len_pages)
    }

    pub fn page_size(&self) -> NonZeroUsize {
        self.page_size
    }

    pub fn max_pages(&self) -> usize {
        self.max_pages
    }

    /// At a given index, store a datum in the [`SharedLinearMemory`]
    pub fn store<const N: usize, T: LittleEndianBytes<N>>(
        &self,
        index: usize,
        value: T,
        ord: Ordering,
    ) -> Result<(), RuntimeError> {
        // 13.
        let index_is_properly_aligned = index.is_multiple_of(N);
        if ord == Ordering::SeqCst && !index_is_properly_aligned {
            return Err(TrapError::UnalignedAtomicAccess.into());
        }

        // 16.
        let bytes = value.to_le_bytes();
        // 18. The current memory is shared
        // a.
        let n = self.rd_len(Ordering::Unord);
        // b.
        let end_index_is_out_of_bounds = index.checked_add(N).is_none_or(|end_index| end_index > n);
        if end_index_is_out_of_bounds {
            return Err(TrapError::MemoryOrDataAccessOutOfBounds.into());
        }
        // c.
        // TODO There is a contradiction: tearing is allowed for types with <= 32 bits. However, 64
        // bit atomic accesses could still be required to not tear as described here:
        // https://github.com/WebAssembly/threads/issues/9#issuecomment-725389629
        let may_tear: bool = tearing::<N, T>(index);

        // d.
        // SAFETY: The end index addition did not overflow and it is not larger than the length `n`
        // that as just read.
        unsafe { self.wr_data(index, bytes.into_iter(), ord, may_tear) };

        Ok(())
    }

    /// At a given index, store a datum in the [`SharedLinearMemory`]
    ///
    /// This operation may tear.
    pub fn store_bytes<const N: usize>(
        &self,
        index: usize,
        bytes: [u8; N],
        ord: Ordering,
    ) -> Result<(), RuntimeError> {
        let end_index = index
            .checked_add(N)
            .ok_or(TrapError::MemoryOrDataAccessOutOfBounds)?;

        let len = self.rd_len(Ordering::Unord);
        if end_index > len {
            return Err(TrapError::MemoryOrDataAccessOutOfBounds.into());
        }

        // SAFETY: calculation of the the end index does not overflow and the end index is not
        // larger than the length that was just read.
        unsafe { self.wr_data(index, bytes.into_iter(), ord, true) };

        Ok(())
    }

    /// From a given index, load a datum from the [`SharedLinearMemory`]
    pub fn load<const N: usize, T: LittleEndianBytes<N>>(
        &self,
        index: usize,
        ord: Ordering,
    ) -> Result<T, RuntimeError> {
        // 13.
        let index_is_properly_aligned = index.is_multiple_of(N);
        if ord == Ordering::SeqCst && !index_is_properly_aligned {
            return Err(TrapError::UnalignedAtomicAccess.into());
        }

        // 18. The current memory is shared
        // a.
        let n = self.rd_len(Ordering::Unord);
        // b.
        let end_index_is_out_of_bounds = index.checked_add(N).is_none_or(|end_index| end_index > n);
        if end_index_is_out_of_bounds {
            return Err(TrapError::MemoryOrDataAccessOutOfBounds.into());
        }
        // c.
        // TODO There is a contradiction: tearing is allowed for types with <= 32 bits. However, 64
        // bit atomic accesses could still be required to not tear as described here:
        // https://github.com/WebAssembly/threads/issues/9#issuecomment-725389629
        let may_tear: bool = tearing::<N, T>(index);

        // d.
        // SAFETY: The end index addition does not overflow and it is not larger than the length `n`
        // that was just read.
        let bytes = unsafe { self.rd_data_into_array::<N>(index, ord, may_tear) };

        Ok(T::from_le_bytes(bytes))
    }

    /// From a given index, load a number of bytes from the [`SharedLinearMemory`]
    ///
    /// This operation may tear.
    pub fn load_bytes<const N: usize>(
        &self,
        index: usize,
        ord: Ordering,
    ) -> Result<[u8; N], RuntimeError> {
        let n = self.rd_len(Ordering::Unord);

        let end_index_is_out_of_bounds = index.checked_add(N).is_none_or(|end_index| end_index > n);
        if end_index_is_out_of_bounds {
            return Err(TrapError::MemoryOrDataAccessOutOfBounds.into());
        }

        // SAFETY: The end index addition did not overflow and it is not larger than the length that
        // was just read.
        Ok(unsafe { self.rd_data_into_array(index, ord, true) })
    }

    /// The `memory.fill` instruction. It sets a number of bytes `count` in this memory to a
    /// specific constant `data_byte`, starting at index `index`.
    ///
    /// Note that the Wasm specification defines this instruction by recursion, while this
    /// implementation uses iteration.
    ///
    /// # See also
    ///
    /// - [WebAssembly Specification 2.0 (with Threads Proposal) - 4.4.7 Memory Instructions - mem.fill](https://webassembly.github.io/threads/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-memory-mathsf-memory-fill)
    pub fn fill(&self, index: usize, data_byte: u8, count: usize) -> Result<(), RuntimeError> {
        let end_index = index
            .checked_add(count)
            .ok_or(TrapError::MemoryOrDataAccessOutOfBounds)?;

        let len = self.rd_len(Ordering::Unord);

        if end_index > len {
            return Err(TrapError::MemoryOrDataAccessOutOfBounds.into());
        }

        let source_buffer_iterator = iter::repeat_n(data_byte, count);

        // SAFETY: The end index did not overflow and is not larger than the length that was just
        // read.
        unsafe { self.wr_data_tearing(index, source_buffer_iterator, Ordering::Unord) };

        Ok(())
    }

    /// The `memory.copy` instruction. It copies a number of bytes `count` from one region in this
    /// memory to another region.  Starting indices must be specified for both the source and
    /// destination regions, which may also overlap.
    ///
    /// Note that the reference specification for this implementation does not support multiple
    /// memories. Therefore, no operation to copy data between multiple memories exists.
    ///
    /// # See also
    ///
    /// - [WebAssembly Specification 2.0 (with Threads Proposal) - 4.4.7 Memory Instructions - mem.copy](https://webassembly.github.io/threads/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-memory-mathsf-memory-copy)
    pub fn copy_within(
        &self,
        destination_index: usize,
        source_index: usize,
        count: usize,
    ) -> Result<(), RuntimeError> {
        let source_end_index = source_index
            .checked_add(count)
            .ok_or(TrapError::MemoryOrDataAccessOutOfBounds)?;
        let destination_end_index = destination_index
            .checked_add(count)
            .ok_or(TrapError::MemoryOrDataAccessOutOfBounds)?;

        let len = self.rd_len(Ordering::Unord);

        if source_end_index > len || destination_end_index > len {
            return Err(TrapError::MemoryOrDataAccessOutOfBounds.into());
        }

        // We have to perform the copy backwards if certain overlaps are possible
        if destination_index < source_index {
            // Forwards
            for offset in 0..count {
                debug_assert!(source_index.checked_add(offset).is_some());
                // SAFETY: The index addition producing the end index does not overflow, it is
                // `source_end_index`.`offset` is also strictly smaller than count, therefore this
                // cannot not overflow.
                let final_source_index = unsafe { source_index.unchecked_add(offset) };

                debug_assert!(destination_index.checked_add(offset).is_some());
                // SAFETY: The index addition producing the end index does not overflow, it is
                // `destination_end_index`. `offset` is also strictly smaller than count, therefore
                // this cannot not overflow.
                let final_destination_index = unsafe { destination_index.unchecked_add(offset) };

                debug_assert!(final_source_index < len);
                debug_assert!(final_destination_index < len);
                // SAFETY: Both the final source and destination indices must be smaller than
                // source_end_index and source_destination index which are smaller than len,
                // respectively.
                unsafe {
                    self.copy_byte(final_source_index, final_destination_index, Ordering::Unord)
                };
            }
        } else {
            // Backwards
            for offset in (0..count).rev() {
                debug_assert!(source_index.checked_add(offset).is_some());
                // SAFETY: The index addition producing the end index does not overflow, it is
                // `source_end_index`.`offset` is also strictly smaller than count, therefore this
                // cannot not overflow.
                let final_source_index = unsafe { source_index.unchecked_add(offset) };

                debug_assert!(destination_index.checked_add(offset).is_some());
                // SAFETY: The index addition producing the end index does not overflow, it is
                // `destination_end_index`. `offset` is also strictly smaller than count, therefore
                // this cannot not overflow.
                let final_destination_index = unsafe { destination_index.unchecked_add(offset) };

                debug_assert!(final_source_index < len);
                debug_assert!(final_destination_index < len);
                // SAFETY: Both the final source and destination indices must be smaller than
                // source_end_index and source_destination index which are smaller than len,
                // respectively.
                unsafe {
                    self.copy_byte(final_source_index, final_destination_index, Ordering::Unord)
                };
            }
        }

        Ok(())
    }

    /// Copies a single byte from a given source index to some destination index.
    ///
    /// # Safety
    ///
    /// `source_index` and `destination_index` must both be less than some length previously
    /// returned by [`Self::rd_len`].
    unsafe fn copy_byte(&self, source_index: usize, destination_index: usize, ord: Ordering) {
        // TODO optimize this. Currently copying one byte acquires the lock twice (rd and wr
        // actions both acquire the lock internally).

        debug_assert!(source_index.checked_add(1).is_some());
        // SAFETY: Caller ensures that `source_index` is less than some length which must therefore
        // be at least `source_index + 1`. Thus `source_index + 1` does not overflow.
        let [byte] = unsafe { self.rd_data_into_array::<1>(source_index, ord, true) };

        debug_assert!(destination_index.checked_add(1).is_some());
        // SAFETY: Caller ensures that `destination_index` is less than some length which must
        // therefore be at least `destination_index + 1`. Thus `destination_index + 1` does not
        // overflow.
        unsafe { self.wr_data_tearing(destination_index, iter::once(byte), ord) };
    }

    // Rationale behind having `source_index` and `count` when the callsite could also just create a
    // subslice for `source_data`? Have all the index error checks in one place.
    //
    // <https://webassembly.github.io/spec/core/exec/instructions.html#xref-syntax-instructions-syntax-instr-memory-mathsf-memory-init-x>
    pub fn init(
        &self,
        destination_index: usize,
        source_data: &[u8],
        source_index: usize,
        count: usize,
    ) -> Result<(), RuntimeError> {
        let source_end_index = source_index
            .checked_add(count)
            .ok_or(TrapError::MemoryOrDataAccessOutOfBounds)?;
        let destination_end_index = destination_index
            .checked_add(count)
            .ok_or(TrapError::MemoryOrDataAccessOutOfBounds)?;

        let source_bytes = source_data
            .get(source_index..source_end_index)
            .ok_or(TrapError::MemoryOrDataAccessOutOfBounds)?;

        let len = self.rd_len(Ordering::Unord);

        if destination_end_index > len {
            return Err(TrapError::MemoryOrDataAccessOutOfBounds.into());
        }

        // SAFETY: The index addition producing the end index does not overflow, it is
        // `destination_end_index`. Also, it was checked that it is not larger than the length of
        // the current memory.
        unsafe {
            self.wr_data_tearing(
                destination_index,
                source_bytes.iter().copied(),
                Ordering::Unord,
            )
        };

        Ok(())
    }

    /// The rd_len action.
    ///
    /// This action returns the length of this shared linear memory at this very specific point in
    /// time. The returned length is not the exact length of the memory, as the memory could have
    /// grown since reading of its length took place.
    ///
    /// Note that shared linear memories can only ever grow. As a result, lengths returned by this
    /// method may only be used as a lower bound for the memory's current size for future
    /// operations.
    ///
    /// # See also
    ///
    /// - [WebAssembly Specification 2.0 (with Threads Proposal) - 4.2.17 Events - act](https://webassembly.github.io/threads/core/exec/runtime.html#syntax-act)
    /// - [`Self::rd_len_in_pages`]
    pub fn rd_len(&self, ord: Ordering) -> usize {
        // We do not use the memory ordering for the length field because it is not an atomic
        // integer and protected by the surrounding lock.
        let _ = ord;

        let lock_guard = self.inner_data.read();
        // Note: The spec defines a `rd` action to have a specific ord, but we can just directly
        // read from the &usize, as no-one has write access to it anyway.
        lock_guard.len()
    }

    /// A version of [`Self::rd_len`] that returns the length of this memory in pages instead of
    /// bytes. Refer to its documentation for more information.
    pub fn rd_len_in_pages(&self, ord: Ordering) -> usize {
        self.rd_len(ord) / self.page_size
    }

    /// The rd_data action.
    ///
    /// This action reads `n` bytes from `index..(index + n)` into a provided buffer where `n` is
    /// the length of that buffer in bytes.
    ///
    /// # See also
    ///
    /// - [WebAssembly Specification 2.0 (with Threads Proposal) - 4.2.17 Events - act](https://webassembly.github.io/threads/core/exec/runtime.html#syntax-act)
    /// - [`Self::rd_data_into_array`]
    ///
    /// # Safety
    ///
    /// - `index + n` must not overflow a usize.
    /// - `index + n` must not be larger than some length previously returned by [`Self::rd_len`].
    ///
    /// `n`: length of destination buffer in bytes.
    pub unsafe fn rd_data(
        &self,
        index: usize,
        dst_byte_buffer: impl AsMut<[MaybeUninit<u8>]>,
        ord: Ordering,
        may_tear: bool,
    ) {
        if may_tear {
            // SAFETY: The caller ensures that `index + n` does not overflow a usize and that it is
            // not larger than a given length.
            unsafe { self.rd_data_tearing(index, dst_byte_buffer, ord) };
        } else {
            // SAFETY: The caller ensures that `index + n` does not overflow a usize and that it is
            // not larger than a given length.
            unsafe { Self::rd_data_no_tears(self.inner_data.write(), index, dst_byte_buffer) };
        }
    }

    /// A version of [`Self::rd_data`] that returns its data via an array.
    ///
    /// # Safety
    ///
    /// - `index + N` must not overflow a usize.
    /// - `index + n` must not be larger than some length previously returned by [`Self::rd_len`].
    pub unsafe fn rd_data_into_array<const N: usize>(
        &self,
        index: usize,
        ord: Ordering,
        may_tear: bool,
    ) -> [u8; N] {
        let mut bytes = [const { MaybeUninit::uninit() }; N];
        // SAFETY: The caller ensures that `index + n` does not overflow a usize and that it is
        // not larger than a given length.
        unsafe { self.rd_data(index, &mut bytes, ord, may_tear) };

        // Workaround for feature `maybe_uninit_uninit_array_transpose`
        bytes.map(|byte| {
            // SAFETY: u8 and MaybeUninit<u8> have the same layout and rd_data guarantees that all
            // bytes passed to it are initialized properly.
            unsafe { byte.assume_init() }
        })
    }

    /// The tearing [`Self::rd_data`] variant.
    ///
    /// # Safety
    ///
    /// - `index + n` must not overflow a usize.
    /// - `index + n` must not be larger than some length previously returned by [`Self::rd_len`].
    ///
    /// `n`: length of destination buffer in bytes.
    unsafe fn rd_data_tearing(
        &self,
        index: usize,
        mut dst_byte_buffer: impl AsMut<[MaybeUninit<u8>]>,
        ord: Ordering,
    ) {
        // We can probably keep the lock guard across the entire loop. Although it might make sense
        // to re-acquire it on every iteration for really large reads.
        let lock_guard = self.inner_data.read();

        for (offset, dst_byte) in dst_byte_buffer.as_mut().iter_mut().enumerate() {
            debug_assert!(index.checked_add(offset).is_some());
            // SAFETY: The caller ensures that `index + n` never overflows. `offset` is strictly
            // smaller than `n`, and thus `index + offset` also cannot overflow.
            let final_index = unsafe { index.unchecked_add(offset) };

            debug_assert!(lock_guard.get(final_index).is_some());
            // SAFETY: The caller ensures that `index + n <= len`. Also i is always smaller than n
            // and thus `index + i` is always within bounds.
            let src = unsafe { lock_guard.get_unchecked(final_index) };

            let byte = src.load(ord.into_atomic_ordering());
            dst_byte.write(byte);
        }
    }

    /// The non-tearing [`Self::rd_data`] variant.
    ///
    /// This method takes a write lock guard as an argument, so that callers can potentially re-use
    /// the lock guard.
    ///
    /// # Safety
    ///
    /// - `index + n` must not overflow a usize.
    /// - `index + n` must not be larger than some length previously returned by [`Self::rd_len`].
    unsafe fn rd_data_no_tears(
        mut lock_guard: WriteLockGuard<'_, Vec<AtomicU8>>,
        index: usize,
        mut dst_byte_buffer: impl AsMut<[MaybeUninit<u8>]>,
    ) {
        for (offset, dst_byte) in dst_byte_buffer.as_mut().iter_mut().enumerate() {
            debug_assert!(index.checked_add(offset).is_some());
            // SAFETY: The caller ensures that `index + n` never overflows. `offset` is strictly
            // smaller than `n`, and thus `index + offset` also cannot overflow.
            let final_index = unsafe { index.unchecked_add(offset) };

            debug_assert!(lock_guard.get(final_index).is_some());
            // SAFETY: The caller ensures that `index + n <= len`. Also i is always smaller than n
            // and thus `index + i` is always within bounds.
            let src = unsafe { lock_guard.get_unchecked_mut(final_index) };

            // Note: We have an exclusive reference, and therefore no need to use an atomic
            // instruction.
            let byte = *src.get_mut();
            dst_byte.write(byte);
        }
    }

    /// The `wr` action
    ///
    /// This action writes `n` bytes into the memory starting at `index`. The bytes are read from a
    /// provided buffer in the form of an [`ExactSizeIterator`] where `n` is its length.
    ///
    /// # See also
    ///
    /// - [WebAssembly Specification 2.0 (with Threads Proposal) - 4.2.17 Events - act](https://webassembly.github.io/threads/core/exec/runtime.html#syntax-act)
    /// - [`Self::rd_data_into_array`]
    ///
    /// # Safety
    ///
    /// - `index + n` must not overflow a usize.
    /// - `index + n` must not be larger than some length previously returned by [`Self::rd_len`].
    ///
    /// `n`: length of source buffer in bytes.
    pub unsafe fn wr_data(
        &self,
        index: usize,
        bytes: impl ExactSizeIterator<Item = u8>,
        ord: Ordering,
        may_tear: bool,
    ) {
        debug_assert!(index.checked_add(bytes.len()).is_some());

        if may_tear {
            // SAFETY: The caller ensures that `index + n` does not overflow a usize and that it is
            // not larger than a given length.
            unsafe { self.wr_data_tearing(index, bytes, ord) }
        } else {
            // SAFETY: The caller ensures that `index + n` does not overflow a usize and that it is
            // not larger than a given length.
            unsafe { Self::wr_data_no_tears(self.inner_data.write(), index, bytes) }
        }
    }

    /// The tearing [`Self::wr_data`] variant.
    ///
    /// # Safety
    ///
    /// - `index + n` must not overflow a usize.
    /// - `index + n` must be less or equal to some length previously returned by [`Self::rd_len`].
    ///
    /// `n`: length of iterator over source buffer bytes.
    unsafe fn wr_data_tearing(
        &self,
        index: usize,
        bytes: impl ExactSizeIterator<Item = u8>,
        ord: Ordering,
    ) {
        let lock_guard = self.inner_data.read();

        for (offset, byte) in bytes.enumerate() {
            debug_assert!(index.checked_add(offset).is_some());
            // SAFETY: The caller ensures that `index + n` never overflows. `offset` is strictly
            // smaller than `n`, and thus `index + offset` also cannot overflow.
            let final_index = unsafe { index.unchecked_add(offset) };

            debug_assert!(lock_guard.get(final_index).is_some());
            // SAFETY: The caller ensures that `index + n <= len`. Also i is always smaller than n
            // and thus `index + i` is always within bounds.
            let dst = unsafe { lock_guard.get_unchecked(final_index) };

            dst.store(byte, ord.into_atomic_ordering());
        }
    }

    /// The non-tearing [`Self::wr_data`] variant.
    ///
    /// # Safety
    ///
    /// - `index + n` must not overflow a usize.
    /// - `index + n` must not be larger than some length previously returned by [`Self::rd_len`].
    ///
    /// `n`: length of iterator over source buffer bytes.
    unsafe fn wr_data_no_tears(
        mut lock_guard: WriteLockGuard<'_, Vec<AtomicU8>>,
        index: usize,
        bytes: impl ExactSizeIterator<Item = u8>,
    ) {
        for (offset, byte) in bytes.enumerate() {
            debug_assert!(index.checked_add(offset).is_some());
            // SAFETY: The caller ensures that `index + n` never overflows. `offset` is strictly
            // smaller than `n`, and thus `index + offset` also cannot overflow.
            let final_index = unsafe { index.unchecked_add(offset) };

            debug_assert!(lock_guard.get(final_index).is_some());
            // SAFETY: The caller ensures that `index + n <= len`. Also i is always smaller than n
            // and thus `index + i` is always within bounds.
            let dst = unsafe { lock_guard.get_unchecked_mut(final_index) };

            // Note: We have an exclusive reference, and therefore no need to use an atomic
            // instruction.
            *dst.get_mut() = byte;
        }
    }

    pub unsafe fn rmw_data_u32_add(&self, properly_aligned_index: usize, a: u32) -> u32 {
        unsafe { self.rmw_data_action(properly_aligned_index, |x| x + a) }
    }

    #[inline(always)]
    unsafe fn rmw_data_action<const N: usize, T: LittleEndianBytes<N> + Copy>(
        &self,
        properly_aligned_index: usize,
        f: impl FnOnce(T) -> T,
    ) -> T {
        let mut lock_guard = self.inner_data.write();

        let atomic_bytes = unsafe {
            lock_guard.get_unchecked_mut(properly_aligned_index..(properly_aligned_index + N))
        };
        let bytes = atomic_u8_get_mut_slice(atomic_bytes);
        let bytes_array: &mut [u8; N] = bytes.try_into().unwrap();

        let value = T::from_le_bytes(*bytes_array);
        let new_value = f(value);
        *bytes_array = new_value.to_le_bytes();

        value
    }

    /// Allows a given closure to temporarily access the entire memory as a `&mut [u8]`.
    ///
    /// # Note on locking
    ///
    /// This operation exclusively locks the entire linear memory temporarily. This operation blocks
    /// until the lock is successfully acquired.
    pub fn access_mut_slice<R>(&self, accessor: impl FnOnce(&mut [u8]) -> R) -> R {
        let mut write_lock_guard = self.inner_data.write();
        let non_atomic_slice = atomic_u8_get_mut_slice(&mut write_lock_guard);
        accessor(non_atomic_slice)
    }
}

impl core::fmt::Debug for SharedLinearMemory {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        /// A helper struct for formatting a [`Vec<UnsafeCell<u8>>`] which is guarded by a [`ReadLockGuard`].
        /// This formatter is able to detect and format byte repetitions in a compact way.
        struct RepetitionDetectingMemoryWriter<'a>(ReadLockGuard<'a, Vec<AtomicU8>>);
        impl core::fmt::Debug for RepetitionDetectingMemoryWriter<'_> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                /// The number of repetitions required for successive elements to be grouped
                // together.
                const MIN_REPETITIONS_FOR_GROUP: usize = 8;

                // First we create an iterator over all bytes
                let mut bytes = self
                    .0
                    .iter()
                    .map(|x| x.load(sync::atomic::Ordering::Relaxed));

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
        f.debug_struct("SharedLinearMemory")
            .field(
                "inner_data",
                &RepetitionDetectingMemoryWriter(self.inner_data.read()),
            )
            .finish()
    }
}

#[inline(always)]
fn tearing<const N: usize, T: LittleEndianBytes<N>>(index: usize) -> bool {
    let is_properly_aligned = index.is_multiple_of(N);
    let is_max_32_bits_large = N <= 4;
    let no_tears = is_properly_aligned && is_max_32_bits_large;
    !no_tears
}

/// Converts an exclusively borrowed slice of atomic `u8`s to a slice of
/// non-atomic `u8`s
// TODO when `atomic_from_mut` is stabilized, replace this function with
// `Atomic::U8::get_mut_slice`
fn atomic_u8_get_mut_slice(slice: &mut [AtomicU8]) -> &mut [u8] {
    // SAFETY: the mutable reference guarantees unique ownership
    unsafe { &mut *(slice as *mut [AtomicU8] as *mut [u8]) }
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
    const MAX_PAGES: usize = 10;

    #[test]
    fn new_constructor() {
        let lin_mem = SharedLinearMemory::new_with_page_size(MAX_PAGES, PAGE_SIZE);
        assert_eq!(lin_mem.rd_len_in_pages(Ordering::SeqCst), 0);
    }

    #[test]
    fn new_grow() {
        let lin_mem = SharedLinearMemory::new_with_page_size(MAX_PAGES, PAGE_SIZE);
        lin_mem.grow::<()>(1).unwrap();
        assert_eq!(lin_mem.rd_len_in_pages(Ordering::SeqCst), 1);
    }

    #[test]
    fn debug_print_simple() {
        let lin_mem =
            SharedLinearMemory::new_with_initial_pages_and_page_size::<()>(1, MAX_PAGES, PAGE_SIZE)
                .unwrap();
        assert_eq!(lin_mem.rd_len_in_pages(Ordering::SeqCst), 1);

        let expected = format!("SharedLinearMemory {{ inner_data: [#{PAGE_SIZE} × 0] }}");
        let debug_repr = format!("{lin_mem:?}");

        assert_eq!(debug_repr, expected);
    }

    #[test]
    fn debug_print_complex() {
        let page_count = 2;
        let lin_mem = SharedLinearMemory::new_with_initial_pages_and_page_size::<()>(
            page_count, MAX_PAGES, PAGE_SIZE,
        )
        .unwrap();
        assert_eq!(lin_mem.rd_len_in_pages(Ordering::SeqCst), page_count);

        lin_mem.store(1, 0xffu8, Ordering::SeqCst).unwrap();
        lin_mem.store(10, 1u8, Ordering::SeqCst).unwrap();
        lin_mem.store(200, 0xffu8, Ordering::Unord).unwrap();

        let expected =
            "SharedLinearMemory { inner_data: [0, 255, #8 × 0, 1, #189 × 0, 255, #311 × 0] }";
        let debug_repr = format!("{lin_mem:?}");

        assert_eq!(debug_repr, expected);
    }

    #[test]
    fn debug_print_empty() {
        let lin_mem =
            SharedLinearMemory::new_with_initial_pages_and_page_size::<()>(0, MAX_PAGES, PAGE_SIZE)
                .unwrap();
        assert_eq!(lin_mem.rd_len_in_pages(Ordering::SeqCst), 0);

        let expected = "SharedLinearMemory { inner_data: [] }";
        let debug_repr = format!("{lin_mem:?}");

        assert_eq!(debug_repr, expected);
    }

    #[test]
    fn roundtrip_normal_range_i8_neg127() {
        let x: i8 = -127;
        let highest_legal_offset = PAGE_SIZE.get() - mem::size_of::<i8>();
        for offset in 0..highest_legal_offset {
            let lin_mem = SharedLinearMemory::new_with_initial_pages_and_page_size::<()>(
                PAGES, MAX_PAGES, PAGE_SIZE,
            )
            .unwrap();

            lin_mem.store(offset, x, Ordering::Unord).unwrap();

            assert_eq!(
                lin_mem
                    .load::<{ core::mem::size_of::<i8>() }, i8>(offset, Ordering::Unord)
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
            let lin_mem = SharedLinearMemory::new_with_initial_pages_and_page_size::<()>(
                PAGES, MAX_PAGES, PAGE_SIZE,
            )
            .unwrap();

            lin_mem.store(offset, x, Ordering::Unord).unwrap();

            assert_eq!(
                lin_mem
                    .load::<{ core::mem::size_of::<F32>() }, F32>(offset, Ordering::Unord)
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
            let lin_mem = SharedLinearMemory::new_with_initial_pages_and_page_size::<()>(
                PAGES, MAX_PAGES, PAGE_SIZE,
            )
            .unwrap();

            lin_mem.store(offset, x, Ordering::Unord).unwrap();

            assert_eq!(
                lin_mem
                    .load::<{ core::mem::size_of::<F64>() }, F64>(offset, Ordering::Unord)
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
            let lin_mem = SharedLinearMemory::new_with_initial_pages_and_page_size::<()>(
                PAGES, MAX_PAGES, PAGE_SIZE,
            )
            .unwrap();

            lin_mem.store(offset, x, Ordering::Unord).unwrap();

            assert!(
                lin_mem
                    .load::<{ core::mem::size_of::<F64>() }, F64>(offset, Ordering::Unord)
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
        let lowest_illegal_offset = PAGE_SIZE.get() - mem::size_of::<u128>() + 1;
        let lin_mem = SharedLinearMemory::new_with_initial_pages_and_page_size::<()>(
            pages, MAX_PAGES, PAGE_SIZE,
        )
        .unwrap();

        lin_mem
            .store(lowest_illegal_offset, x, Ordering::Unord)
            .unwrap();
    }

    #[test]
    #[should_panic(
        expected = "called `Result::unwrap()` on an `Err` value: Trap(MemoryOrDataAccessOutOfBounds)"
    )]
    fn store_empty_lineaer_memory_u8() {
        let x: u8 = u8::MAX;
        let pages = 0;
        let lowest_illegal_offset = PAGE_SIZE.get() - mem::size_of::<u8>() + 1;
        let lin_mem = SharedLinearMemory::new_with_initial_pages_and_page_size::<()>(
            pages, MAX_PAGES, PAGE_SIZE,
        )
        .unwrap();

        lin_mem
            .store(lowest_illegal_offset, x, Ordering::Unord)
            .unwrap();
    }

    #[test]
    #[should_panic(
        expected = "called `Result::unwrap()` on an `Err` value: Trap(MemoryOrDataAccessOutOfBounds)"
    )]
    fn load_out_of_range_u128_max() {
        let pages = 1;
        let lowest_illegal_offset = PAGE_SIZE.get() - mem::size_of::<u128>() + 1;
        let lin_mem = SharedLinearMemory::new_with_initial_pages_and_page_size::<()>(
            pages, MAX_PAGES, PAGE_SIZE,
        )
        .unwrap();

        let _x: u128 = lin_mem
            .load(lowest_illegal_offset, Ordering::Unord)
            .unwrap();
    }

    #[test]
    #[should_panic(
        expected = "called `Result::unwrap()` on an `Err` value: Trap(MemoryOrDataAccessOutOfBounds)"
    )]
    fn load_empty_lineaer_memory_u8() {
        let pages = 0;
        let lowest_illegal_offset = PAGE_SIZE.get() - mem::size_of::<u8>() + 1;
        let lin_mem = SharedLinearMemory::new_with_initial_pages_and_page_size::<()>(
            pages, MAX_PAGES, PAGE_SIZE,
        )
        .unwrap();

        let _x: u8 = lin_mem
            .load(lowest_illegal_offset, Ordering::Unord)
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn copy_out_of_bounds() {
        let lin_mem_0 =
            SharedLinearMemory::new_with_initial_pages_and_page_size::<()>(2, MAX_PAGES, PAGE_SIZE)
                .unwrap();
        lin_mem_0
            .copy_within(PAGE_SIZE.get(), 0, PAGE_SIZE.get() + 1)
            .unwrap();
    }
}
