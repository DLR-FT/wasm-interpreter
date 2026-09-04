use core::convert::Infallible;

use alloc::boxed::Box;

#[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
pub trait ToUsizeExt {
    fn into_usize(self) -> usize;
}

#[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
impl ToUsizeExt for u32 {
    fn into_usize(self) -> usize {
        // SAFETY: The current trait only exists for architectures where
        // pointers are at least 32 bits wide.
        unsafe { usize::try_from(self).unwrap_unchecked() }
    }
}

#[cfg(target_pointer_width = "16")]
compile_error!("targets with 16 bit wide pointers are currently not supported");

pub trait MapIntoBoxedSlice<T> {
    fn map_into_boxed_slice<F: FnMut(&T) -> R, R>(&self, mapper: F) -> Box<[R]>;

    fn try_map_into_boxed_slice<F: FnMut(&T) -> Result<R, E>, R, E>(
        &self,
        mapper: F,
    ) -> Result<Box<[R]>, E>;
}

impl<T> MapIntoBoxedSlice<T> for [T] {
    /// A helper for mapping each element in a slice to some other type. The resulting elements are
    /// allocated as a boxed slice.
    ///
    /// While that is basically equivalent to `slice.iter().map(...).collect()`, this function
    /// guarantees that only a single allocation happens, without any re- or deallocations. Also it
    /// can easily support custom allocators whereas `collect` cannot.
    ///
    /// See [`MapIntoBoxedSlice::try_map_into_boxed_slice`] when you want to use a fallible mapper
    /// function.
    fn map_into_boxed_slice<F: FnMut(&T) -> R, R>(&self, mut mapper: F) -> Box<[R]> {
        let result = Self::try_map_into_boxed_slice(&self, |t| Ok::<R, Infallible>(mapper(t)));

        debug_assert!(result.is_ok());
        // SAFETY: The error type is infallible.
        unsafe { result.unwrap_unchecked() }
    }

    /// Note: If the mapper function panics, all initialized `R`s are never dropped, i.e. leaked.
    fn try_map_into_boxed_slice<F: FnMut(&T) -> Result<R, E>, R, E>(
        &self,
        mut mapper: F,
    ) -> Result<Box<[R]>, E> {
        let len = self.len();
        let mut new_slice = Box::new_uninit_slice(len);

        for i in 0..len {
            debug_assert!(self.get(i).is_some());
            // SAFETY: `i` is always less than `len`.
            let t = unsafe { self.get_unchecked(i) };

            debug_assert!(new_slice.get(i).is_some());
            // SAFETY: `i` is always less than `len`.
            let slot = unsafe { new_slice.get_unchecked_mut(i) };

            slot.write(mapper(t)?);
        }

        // SAFETY: All `len` elements have been initialized in the for-loop.
        Ok(unsafe { new_slice.assume_init() })
    }
}
