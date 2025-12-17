//! # Type-safe Indices and Index Spaces
//!
//! Wasm specifies different classes of definitions (types, functions, globals,
//! ...). Each definition can be uniquely addressed by a single index
//! represented as a 32-bit integer. For every definition class, there exists a
//! separate index space, along with a special index type per class: `typeidx` for
//! types, `funcidx` for functions, etc.
//!
//! Using `u32` and `Vec` types to represent such indices and index spaces
//! across all classes of definitions comes with risks.
//!
//! This module defines one newtype index type per definition class (e.g.
//! [`TypeIdx`], [`FuncIdx`], [`GlobalIdx`]) and an index space [`IdxVec`].
//!
//!
//! # What this module is not
//!
//! However, this module cannot ensure that an index type is always used for the
//! [`IdxVec`] it originally came from. Imagine a scenario, in which multiple
//! Wasm modules are used. Even though it would not make any sense to use
//! indices across multiple index spaces of the same definition class, detecting
//! such mis-use is out of the scope of this module.
//!
//! Instead, the caller must guarantee that indices are only used together with
//! the correct index space. These guarantees are documented in the form of
//! safety requirements.
//!
//! See: WebAssembly Specification - 2.5.1 - Indices

use core::marker::PhantomData;

use alloc::{boxed::Box, vec::Vec};

/// A trait for all index types.
///
/// This is used by [`IdxVec`] to create and read index types.
#[allow(unused)] // reason = "temporary until used by new index types"
pub trait Idx: Copy + core::fmt::Debug + core::fmt::Display + Eq {
    fn new(index: u32) -> Self;

    fn into_inner(self) -> u32;
}

/// An immutable vector that can only be indexed by type-safe 32-bit indices.
///
/// Use [`IdxVec::new`] or [`IdxVec::default`] to create a new instance.
#[allow(unused)] // reason = "temporary until used by new index types"
pub struct IdxVec<I: Idx, T> {
    inner: Box<[T]>,
    _phantom: PhantomData<I>,
}

impl<I: Idx, T> Default for IdxVec<I, T> {
    fn default() -> Self {
        Self {
            inner: Box::default(),
            _phantom: PhantomData,
        }
    }
}

impl<I: Idx, T: Clone> Clone for IdxVec<I, T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<I: Idx, T: core::fmt::Debug> core::fmt::Debug for IdxVec<I, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(&*self.inner).finish()
    }
}

/// An error for when an [`IdxVec`] is initialized with too many elements.
#[derive(Debug)]
pub struct IdxVecOverflowError;

impl<I: Idx, T> IdxVec<I, T> {
    /// Creates a new [`IdxVec`] with the given elements in it.
    ///
    /// If the number of elements is larger than what can be addressed by a
    /// `u32`, i.e. `u32::MAX`, an error is returned instead.
    #[allow(unused)] // reason = "temporary until used by new index types"
    pub fn new(elements: Vec<T>) -> Result<Self, IdxVecOverflowError> {
        if u32::try_from(elements.len()).is_err() {
            return Err(IdxVecOverflowError);
        }

        Ok(Self {
            inner: elements.into_boxed_slice(),
            _phantom: PhantomData,
        })
    }

    #[allow(unused)] // reason = "temporary until used by new index types"
    fn validate_index(&self, index: u32) -> Option<I> {
        let index_as_usize = usize::try_from(index).expect("architecture to be at least 32 bits");
        let _element = self.inner.get(index_as_usize)?;
        Some(I::new(index))
    }

    /// Gets an element from this vector by its index.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the index object was validated using the
    /// same vector as `self` or a different vector used to create `self`
    /// through [`IdxVec::map`].
    #[allow(unused)] // reason = "temporary until used by new index types"
    pub unsafe fn get(&self, index: I) -> &T {
        let index =
            usize::try_from(index.into_inner()).expect("architecture to be at least 32 bits");

        // TODO use `unwrap_unchecked` when we are sure everything is sound and
        // our validation is properly tested
        self.inner
            .get(index)
            .expect("this to be a valid index due to the safety guarantees made by the caller")
    }

    #[allow(unused)] // reason = "temporary until used by new index types"
    pub fn len(&self) -> u32 {
        u32::try_from(self.inner.len()).expect(
            "this to never be larger than u32::MAX, because this was checked for in Self::new",
        )
    }
}

pub type TypeIdx = usize;
pub type FuncIdx = usize;
pub type TableIdx = usize;
pub type MemIdx = usize;
pub type GlobalIdx = usize;
#[allow(dead_code)]
pub type ElemIdx = usize;
pub type DataIdx = usize;
pub type LocalIdx = usize;
pub type LabelIdx = usize;
