use alloc::{boxed::Box, slice};
use core::{
    convert::Infallible,
    hint::unreachable_unchecked,
    iter::{
        Cloned, Copied, Enumerate, Fuse, Inspect, Map, Once, OnceWith, Peekable, Repeat, RepeatN,
        RepeatWith, Rev, Take, Zip,
    },
};

use crate::core::fixed_capacity_vec::FixedCapacityVec;

pub use trusted_chain::*;

/// An iterator that reports an accurate and exact length using size_hint.
///
/// It is exact because it ensures that the lower bound is equal to the upper bound.
///
/// There is one exception: If an iterator returns [`Result`]s, it may be shorter.
///
/// # Safety
///
/// When implemented for some type, its [`Iterator::size_hint`] implementation must return `(n,
/// Some(n))` for some `n`. Also, the iterator must produce a total of `n` `Some(_)` values and then
/// return `None`.
pub unsafe trait TrustedExactSizeIterator: Iterator {
    #[inline]
    fn len_trusted(&self) -> usize {
        // We cannot simply call `ExactSizeIterator::len`, because it would introduce an assert_eq
        // for every call.

        let (lower, upper) = self.size_hint();
        if Some(lower) != upper {
            if cfg!(debug_assertions) {
                panic!("TrustedExactSizeIterator returned unexpected size hint");
            }
            unsafe { unreachable_unchecked() }
        }
        lower
    }
}

/// SAFETY: Slices always have an exact length.
unsafe impl<T> TrustedExactSizeIterator for slice::Iter<'_, T> {}
/// SAFETY: Slices always have an exact length.
unsafe impl<T> TrustedExactSizeIterator for slice::IterMut<'_, T> {}
/// SAFETY: [`RepeatN`]'s entire purpose is to have an exact length of `N`.
unsafe impl<A: Clone> TrustedExactSizeIterator for RepeatN<A> where Self: Iterator {}
unsafe impl<A> TrustedExactSizeIterator for Take<Repeat<A>> where Self: Iterator {}
unsafe impl<F> TrustedExactSizeIterator for Take<RepeatWith<F>> where Self: Iterator {}
unsafe impl<T> TrustedExactSizeIterator for Once<T> {}
unsafe impl<F> TrustedExactSizeIterator for OnceWith<F> where OnceWith<F>: Iterator {}
// TODO: No such thing as `TrustedStep` which core uses. But we do not need a lot of different range
// types.
unsafe impl TrustedExactSizeIterator for core::ops::Range<usize> {}
unsafe impl TrustedExactSizeIterator for core::ops::Range<u32> {}
unsafe impl<T> TrustedExactSizeIterator for alloc::vec::IntoIter<T> {}

unsafe impl<I: TrustedExactSizeIterator> TrustedExactSizeIterator for Cloned<I> where Self: Iterator {}
unsafe impl<I: TrustedExactSizeIterator> TrustedExactSizeIterator for Copied<I> where Self: Iterator {}
unsafe impl<I: TrustedExactSizeIterator> TrustedExactSizeIterator for Enumerate<I> {}
unsafe impl<I: TrustedExactSizeIterator, F> TrustedExactSizeIterator for Map<I, F> where
    Self: Iterator
{
}
unsafe impl<I: TrustedExactSizeIterator> TrustedExactSizeIterator for Fuse<I> {}
unsafe impl<I: TrustedExactSizeIterator, F> TrustedExactSizeIterator for Inspect<I, F> where
    Self: Iterator
{
}
unsafe impl<I: TrustedExactSizeIterator> TrustedExactSizeIterator for Rev<I> where Self: Iterator {}
// Note: This should be fine unless someone could construct a Peekable with an element in its
// `peeked` field and an iterator with length `usize::MAX`, which should not be possible. Therefore,
// this implementation might be considered unsound.
unsafe impl<I: TrustedExactSizeIterator> TrustedExactSizeIterator for Peekable<I> {}
unsafe impl<A: TrustedExactSizeIterator, B: TrustedExactSizeIterator> TrustedExactSizeIterator
    for Zip<A, B>
{
}

pub trait CollectIntoBoxedSliceExt: TrustedExactSizeIterator {
    fn collect_into_boxed_slice(self) -> Box<[Self::Item]>;
}

pub trait TryCollectIntoBoxedSliceExt<T, E>: TrustedExactSizeIterator<Item = Result<T, E>> {
    fn try_collect_into_boxed_slice(self) -> Result<Box<[T]>, E>;
}

impl<I: TrustedExactSizeIterator> CollectIntoBoxedSliceExt for I {
    #[inline]
    fn collect_into_boxed_slice(self) -> Box<[Self::Item]> {
        // Implemented via `try_collect_into_boxed_slice` with E=Infallible

        let result = self
            .map(|t| Ok::<Self::Item, Infallible>(t))
            .try_collect_into_boxed_slice();

        debug_assert!(result.is_ok());
        // SAFETY: The error type is infallible.
        unsafe { result.unwrap_unchecked() }
    }
}

impl<T, E, I> TryCollectIntoBoxedSliceExt<T, E> for I
where
    I: TrustedExactSizeIterator<Item = Result<T, E>>,
{
    #[inline]
    fn try_collect_into_boxed_slice(self) -> Result<Box<[T]>, E> {
        let len = self.len_trusted();
        // We collect the elements in a fixed capacity vec to get a Drop implementation for free for
        // the partially-initialized elements in case of a panic or error.
        //
        // With enough inlining the compiler should even be able to prove the absence of panics or
        // errors in specific cases, possibly improving any overhead introduced here.
        let mut elements = FixedCapacityVec::with_capacity(len);

        for result in self {
            // Return and drop the vec on the first error
            let t = result?;
            debug_assert!(!elements.is_full());
            // SAFETY: The vector has the capacity `len` and the for loop only iterates up to `len`
            // times, so there is always space.
            unsafe { elements.push_unchecked(t) };
        }

        debug_assert!(elements.is_full());
        // SAFETY: A total of `len` elements have been pushed into the vector, so it must be full.
        Ok(unsafe { elements.into_boxed_slice() })
    }
}

pub trait AssumeTrustedExactSize: Iterator {
    unsafe fn assume_trusted_exact_size(
        self,
        exact_size: usize,
    ) -> impl TrustedExactSizeIterator<Item = Self::Item>;
}

struct TrustedExactSizeUnchecked<I> {
    inner: I,
    size: usize,
}

impl<I: Iterator> Iterator for TrustedExactSizeUnchecked<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.size, Some(self.size))
    }
}

unsafe impl<I> TrustedExactSizeIterator for TrustedExactSizeUnchecked<I> where Self: Iterator {}

impl<I: Iterator> AssumeTrustedExactSize for I {
    unsafe fn assume_trusted_exact_size(
        self,
        exact_size: usize,
    ) -> impl TrustedExactSizeIterator<Item = Self::Item> {
        TrustedExactSizeUnchecked {
            inner: self,
            size: exact_size,
        }
    }
}

// Chain requires some special handling due to the addition of lengths it performs.
mod trusted_chain {
    use core::iter::Chain;

    use crate::core::trusted_exact_size_iterator::TrustedExactSizeIterator;

    /// A stricter variant of [`Chain`] with a known exact size.
    ///
    /// The normal chain instead sets the upper bound to `None`, which is incompatible with the
    /// [`TrustedExactSizeIterator`].
    pub struct TrustedChain<A, B>(Chain<A, B>);

    impl<A, B> Iterator for TrustedChain<A, B>
    where
        A: TrustedExactSizeIterator,
        B: TrustedExactSizeIterator<Item = A::Item>,
    {
        type Item = A::Item;

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            self.0.next()
        }

        #[inline]
        fn size_hint(&self) -> (usize, Option<usize>) {
            self.0.size_hint()
        }
    }

    unsafe impl<A, B> TrustedExactSizeIterator for TrustedChain<A, B>
    where
        A: TrustedExactSizeIterator,
        B: TrustedExactSizeIterator<Item = A::Item>,
    {
    }

    pub trait IntoTrustedChainExt<A, B> {
        fn try_into_trusted(self) -> Option<TrustedChain<A, B>>;
    }

    impl<A, B> IntoTrustedChainExt<A, B> for Chain<A, B>
    where
        A: TrustedExactSizeIterator,
        B: TrustedExactSizeIterator<Item = A::Item>,
    {
        #[inline]
        fn try_into_trusted(self) -> Option<TrustedChain<A, B>> {
            // We can create a trusted iterator only if the upper size is known. The lower size will
            // then always be equal to it.
            self.size_hint().1.is_some().then_some(TrustedChain(self))
        }
    }
}
