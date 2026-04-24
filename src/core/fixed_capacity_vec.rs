use core::mem::MaybeUninit;

use alloc::boxed::Box;

use crate::RuntimeError;

pub struct FixedCapacityVec<T> {
    /// A contiguous, non-reallocating, heap allocated vector. Behaves like a subset of
    /// [`alloc::vec::Vec`], cleansed of any operation that reallocates. Backed by via boxed
    /// slice containing elements of type `MaybeUninit<T>`. The maximum size (pendant to
    /// [`alloc::vec::Vec::capacity`]) is determined at creation and remains unchanged over the
    /// lifetime of [`Self`]. Nonetheless a variable amount of `T` can be kept within [`Self`], just
    /// never more than the maximum size passed to the constructor.
    ///
    /// This can be used as a stack, in which case the first element (forming the bottom of the
    /// stack) is at index 0, and new elements grow towards [`Self`]'s end. The last element
    /// (`self.stack[self.stack.len() - 1]` is the topmost possible entry.
    elements: Box<[MaybeUninit<T>]>,

    /// Number of elements in [`Self`]. Also an index, pointing always to the first unused slot in
    /// [`Self::elements`].
    // Developer notes:
    // - It is UB to access `self.elements[self.len]` or any other index larger than `self.len`.
    // - If `self.stack_height == 0`, then there are no elements.
    // - The guaranteed to be initialized range of [`Self::stack`] always is `self.stack[..self.stack_height]`.
    len: usize,
}

impl<T> FixedCapacityVec<T> {
    /// Construct new [`Self`], holding up to `capacity` elements of type `T`
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            elements: Box::new_uninit_slice(capacity),
            len: 0,
        }
    }

    /// Check if the [`Self`] is empty
    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Check if the [`Self`] is full
    #[inline(always)]
    pub const fn is_full(&self) -> bool {
        self.len == self.elements.len()
    }

    /// Get the [`Self`] is height.
    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Get the maximum number of elements that fit into [`Self`].
    #[inline(always)]
    pub const fn capacity(&self) -> usize {
        self.elements.len()
    }

    /// Push a value to the end of [`Self`]
    ///
    /// # Safety
    ///
    /// - Causes UB if [`Self`] is already full.
    /// - Causes UB if [`Self`] has a capacity equal to `usize::MAX`
    #[inline(always)]
    pub unsafe fn push_unchecked(&mut self, value: T) {
        debug_assert!(!self.is_full());
        debug_assert!(self.capacity() < usize::MAX);

        // SAFETY: This must never be called when `Self` is full
        unsafe { *self.elements.get_unchecked_mut(self.len) = MaybeUninit::new(value) };

        // SAFETY: This must never be called when `self.capacity()` is `usize::MAX`
        self.len = unsafe { self.len.unchecked_add(1) };
    }

    /// Push a value to the end of [`Self`]
    pub fn push(&mut self, value: T) -> Result<(), RuntimeError> {
        // check if insertion will overflow `self.len`
        let new_len = self
            .len
            .checked_add(1)
            .ok_or(RuntimeError::StackExhaustion)?;

        *self
            .elements
            .get_mut(self.len)
            .ok_or(RuntimeError::StackExhaustion)? = MaybeUninit::new(value);

        // only "commit" the `new_len` if the insertion actually succeeded
        self.len = new_len;

        Ok(())
    }

    /// Pop a value from [`Self`]'s top/tail
    ///
    /// # Safety
    ///
    /// Causes UB if the stack is empty.
    #[inline(always)]
    pub unsafe fn pop_unchecked(&mut self) -> T {
        debug_assert!(!self.is_empty());

        // SAFETY: This must never be called when `self` is empty. If the stack is indeed not
        // empty, then both the unchecked subtraction is guaranteed not to underflow.
        self.len = unsafe { self.len.unchecked_sub(1) };

        // SAFETY: This is guaranteed to be initialized, as `self.len` accurately tracks the number
        // of initialized values starting from `self.element`'s beginning. As we require `self.len`
        // not to be zero before calling this function, it is now at zero and thus a valid index to
        // an initialized element.
        unsafe { self.elements.get_unchecked(self.len).assume_init_read() }
    }

    /// Pop a value from [`Self`]'s top/tail
    pub fn pop(&mut self) -> Result<T, RuntimeError> {
        self.len = self
            .len
            .checked_sub(1)
            .ok_or(RuntimeError::StackExhaustion)?;

        let value = self
            .elements
            .get(self.len)
            .ok_or(RuntimeError::StackExhaustion)?;

        Ok(
            // SAFETY: This is guaranteed to be initialized, as `self.len` accurately tracks the
            // number of initialized values starting from `self.element`'s beginning.
            unsafe { value.assume_init_read() },
        )
    }

    /// Peek at the topmost/last value from [`Self`]
    ///
    /// # Safety
    ///
    /// Causes UB if the stack is empty.
    #[inline(always)]
    pub unsafe fn peek_unchecked(&self) -> &T {
        debug_assert!(!self.is_empty());

        // SAFETY: This must never be called when the `self` is empty. If the stack is indeed not
        // empty, then both the unchecked subtraction can not underflow, and the element at that
        // index will be already initialized.
        unsafe {
            self.elements
                .get_unchecked(self.len.unchecked_sub(1))
                .assume_init_ref()
        }
    }

    /// Peek at the topmost/last value from [`Self`]
    pub fn peek(&self) -> Result<&T, RuntimeError> {
        let idx = self
            .len
            .checked_sub(1)
            .ok_or(RuntimeError::StackExhaustion)?;

        let value = self
            .elements
            .get(idx)
            .ok_or(RuntimeError::StackExhaustion)?;

        Ok(
            // SAFETY: This is guaranteed to be initialized, as `self.len` accurately tracks the
            // number of initialized values starting from `self.element`'s beginning.
            unsafe { value.assume_init_ref() },
        )
    }

    /// Get a shared ref to the nth element in [`Self`]
    #[inline(always)]
    pub fn get(&self, idx: usize) -> Option<&T> {
        if idx < self.len {
            self.elements.get(idx).map(|e|
            // SAFETY: This is guaranteed to be initialized, as `self.len` accurately tracks the
            // number of initialized values starting from `self.element`'s beginning. The pervious
            // if condition in term checks that idx is Smaller than 
            unsafe{ e.assume_init_ref() })
        } else {
            None
        }
    }

    /// Get a mut ref to the nth element in [`Self`]
    #[inline(always)]
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        if idx < self.len {
            self.elements.get_mut(idx).map(|e|
                // SAFETY: per the previous if statement, the index points into the range of
                // initialized stack members.
                unsafe{
                    e.assume_init_mut()
            })
        } else {
            None
        }
    }
}

impl<T: Clone> FixedCapacityVec<T> {
    /// Push from a slice into [`Self`], appending after the last/topmost element
    pub fn push_from_slice(&mut self, values: &[T]) -> Result<(), RuntimeError> {
        // verify `values` fits into the new self
        if values.len() > (self.elements.len() - self.len) {
            return Err(RuntimeError::StackExhaustion);
        }

        for e in values {
            // SAFETY: this is safe, as the previous if statement verifies that there is enough
            // space for all elements from `values` in `stack`
            unsafe {
                self.push_unchecked(e.clone());
            }
        }

        Ok(())
    }

    /// Pop `n` elements from [`Self`] into a slice. The topmost/last element of [`Self`] will
    /// become the slice's last element.
    pub fn pop_into_slice(
        &mut self,
        n: usize,
    ) -> Result<impl core::ops::Deref<Target = [T]> + '_, RuntimeError> {
        // verify at least `n` elements are still one the stack
        if n > self.elements.len() {
            return Err(RuntimeError::StackExhaustion);
        }

        let old_len = self.len();

        // SAFETY: the previous if statement ensure that `n` is smaller than or equal to
        // `self.elements.len()`, hence this subtraction is guaranteed to not underflow
        self.len = unsafe { self.len.unchecked_sub(n) };

        // SAFETY: the prior if statement checks that `stack_height - n` wont underflow. And the
        // rest of the module ensures that stack_height never gets bigger than `self.stack.len()`.
        // Thus the index range is always in-bounds.
        Ok(SliceDropGuard(unsafe {
            self.elements.get_unchecked_mut(self.len..old_len)
        }))
    }
}

pub struct SliceDropGuard<'a, T>(&'a mut [MaybeUninit<T>]);

impl<'a, T> core::ops::Deref for SliceDropGuard<'a, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        // SAFETY: The rest of the module ensures that stack_height never gets bigger than
        // `self.stack.len()`. Thus the index range is always in-bounds.
        unsafe { slice_assume_init(self.0) }
    }
}

impl<'a, T> Drop for SliceDropGuard<'a, T> {
    fn drop(&mut self) {
        for e in &mut *self.0 {
            // SAFETY: The rest of the module ensures that stack_height never gets bigger than
            // `self.stack.len()`. Thus the index range is always in-bounds.
            unsafe { e.assume_init_drop() };
        }
    }
}

impl<T: Copy> FixedCapacityVec<T> {
    /// Remove `remove_count` values from [`Self`], keeping the topmost `keep_count` values
    ///
    /// From the [`Self`], remove `remove_count` elements, by sliding down the `keep_count` last/topmost
    /// values `remove_count` positions forward/towards the bottom.
    ///
    /// **Effects**
    ///
    /// - after the operation, [`Self`] will contain `remove_count` fewer elements
    /// - `keep_count` topmost elements will be identical before and after the operation
    /// - all elements below the `remove_count + keep_count` topmost stack entry remain
    pub fn remove_in_between(&mut self, remove_count: usize, keep_count: usize) {
        // TODO make unchecked version, remove overflowing arithmetic in safe version
        let len = self.len();
        self.elements
            .copy_within(len - keep_count..len, len - keep_count - remove_count);
        self.len -= remove_count;
    }
}

impl<T: core::fmt::Debug> core::fmt::Debug for FixedCapacityVec<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // SAFETY: per the guarantees of push/pop/etc., `0..self.stack_height` is always a range to
        // initalized values.
        let stack_elements_slice = unsafe { slice_assume_init(&self.elements[..self.len]) };

        f.debug_struct("Stack")
            .field("stack", &stack_elements_slice)
            .field("stack_height", &self.len)
            .finish()
    }
}

impl<T> Drop for FixedCapacityVec<T> {
    fn drop(&mut self) {
        // SAFETY: self is guaranteed to have at least `self.len` elements, because `sel.len` can
        // not grow past `self.elements.len` (aka `self.capacity()`).
        for e in unsafe { self.elements.get_unchecked_mut(0..self.len) } {
            // SAFETY: This is guaranteed to be initialized, as `self.len` accurately tracks the
            // number of initialized values starting from `self.element`'s beginning.
            unsafe { e.assume_init_drop() }
        }
    }
}

// TODO use assume_init_ref, once we get the MSRV to 1.93.0
/// # Safety
///
/// Use this to assume a range of a slice to be initialized. This is provided starting with Rust
/// 1.93.0.
#[inline(always)]
unsafe fn slice_assume_init<T>(slice: &[MaybeUninit<T>]) -> &[T] {
    // SAFETY: casting `slice` to a `*const [T]` is safe since the caller guarantees that
    // `slice` is initialized, and `MaybeUninit` is guaranteed to have the same layout as `T`.
    // The pointer obtained is valid since it refers to memory owned by `slice` which is a
    // reference and thus guaranteed to be valid for reads.
    unsafe { &*(slice as *const _ as *const [T]) }
}
