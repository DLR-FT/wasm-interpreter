use core::convert::Infallible;

use alloc::boxed::Box;

use crate::core::fixed_capacity_vec::FixedCapacityVec;

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
    #[must_use = "allocates in most cases"]
    fn map_into_boxed_slice<F: FnMut(&T) -> R, R>(&self, mapper: F) -> Box<[R]>;

    #[must_use = "allocates in most cases"]
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
    /// Use [`MapIntoBoxedSlice::try_map_into_boxed_slice`] if you need to use a fallible mapper
    /// function.
    fn map_into_boxed_slice<F, R>(&self, mut mapper: F) -> Box<[R]>
    where
        F: FnMut(&T) -> R,
    {
        let result: Result<_, Infallible> = self.try_map_into_boxed_slice(|t| Ok(mapper(t)));

        debug_assert!(result.is_ok());
        // SAFETY: The error type is infallible.
        unsafe { result.unwrap_unchecked() }
    }

    /// Note: If the mapper function panics, all initialized `R`s are never dropped, i.e. leaked.
    fn try_map_into_boxed_slice<F, R, E>(&self, mut mapper: F) -> Result<Box<[R]>, E>
    where
        F: FnMut(&T) -> Result<R, E>,
    {
        let len = self.len();
        // We collect the elements in a fixed capacity vec to get a Drop implementation for free for
        // the partially-initialized elements in case of a panic or error.
        //
        // With enough inlining the compiler should even be able to prove the absence of panics or
        // errors in specific cases, possibly improving any overhead introduced here.
        let mut new_elements = FixedCapacityVec::with_capacity(len);

        for t in self {
            let r = mapper(t)?;

            debug_assert!(!new_elements.is_full());
            // SAFETY: The vector has the capacity `len` and the for loop only iterates up to `len`
            // times, so there is always space.
            unsafe { new_elements.push_unchecked(r) };
        }

        debug_assert!(new_elements.is_full());
        // SAFETY: A total of `len` elements have been pushed into the vector, so it must be full.
        Ok(unsafe { new_elements.into_boxed_slice() })
    }
}

#[cfg(test)]
mod tests {
    use core::{cell::RefCell, fmt::Write};

    use alloc::{rc::Rc, string::String};

    use crate::core::utils::MapIntoBoxedSlice;

    #[test]
    fn map_slice() {
        let input: &[u32] = &[1, 2, 3, 4, 5];

        let output = input.map_into_boxed_slice(|num| u64::from(2 * num));

        assert_eq!(&*output, &[2, 4, 6, 8, 10]);
    }

    #[test]
    fn try_map_slice_ok() -> Result<(), ()> {
        let input: &[u8] = &[1, 2, 4, 32];

        let output = input.try_map_into_boxed_slice(|num| num.checked_mul(2).ok_or(()))?;

        assert_eq!(&*output, &[2, 4, 8, 64]);

        Ok(())
    }

    #[test]
    fn try_map_slice_err() {
        let input: &[u8] = &[1, 2, 4, 32];

        // 32 * 8 will overflow
        let result = input.try_map_into_boxed_slice(|num| num.checked_mul(8).ok_or(()));

        assert_eq!(result, Err(()));
    }

    #[test]
    fn try_map_slice_drop_initialized() {
        let drop_log: Rc<RefCell<String>> = Rc::default();

        let input: &[u32] = &[1, 2, 3, 4, 5];

        struct OutputElement {
            data: u64,
            logger: Rc<RefCell<String>>,
        }

        impl Drop for OutputElement {
            fn drop(&mut self) {
                let mut logger = self.logger.borrow_mut();
                writeln!(&mut *logger, "Dropping {}", self.data).unwrap();
            }
        }

        let result = input.try_map_into_boxed_slice(|num| {
            if *num == 4 {
                return Err(());
            }

            Ok(OutputElement {
                data: u64::from(*num),
                logger: drop_log.clone(),
            })
        });

        assert!(result.is_err());

        let log = Rc::into_inner(drop_log).unwrap().into_inner();
        assert_eq!(log, "Dropping 1\nDropping 2\nDropping 3\n");
    }
}
