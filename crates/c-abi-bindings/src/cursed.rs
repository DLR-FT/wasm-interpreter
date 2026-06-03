//! A module containing cursed helpers.
//!
//! To be reworked.
#[inline(always)]
pub(crate) fn alloc<T>(new_t: T) -> &'static mut T {
    use alloc::boxed::Box;
    Box::leak(Box::new(new_t))
}

/// # Safety
///
/// You must ensure that `ptr` was derived from a call to [`alloc`], and not deallocated before.
/// Also ensure that `ptr` is not used any more after its deallocation.
#[inline(always)]
pub(crate) fn dealloc<T>(ptr: *mut T) {
    use alloc::boxed::Box;
    // SAFETY: It is assumed that `ptr` is a pointer derived from `cursed::alloc`
    unsafe {
        drop(Box::from_raw(ptr));
    }
}
