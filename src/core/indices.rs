//! # Type-state indices and index spaces
//!
//! Wasm uses different classes of definitions such as types, functions,
//! globals, elements, etc. To be able to reference specific definitions inside
//! a specific Wasm module, indices are used.
//!
//! All indices are represented by a [`u32`], however a separate index space is
//! used for each class of definitions (e.g. typeidx for types, funcidx for
//! functions, etc.).
//!
//! A trivial solution would be to directly use [`u32`]s for all index types,
//! however this is error-prone as the developer has to manually make sure not
//! to mis-use of one index space for another.
//!
//! # What this module is
//!
//! This module defines a generic [`IdxVec`] to represent an index space for
//! definitions of a specific class. Also, newtype structs are provided for
//! every class of definitions (e.g. [`TypeIdx`], [`FuncIdx`], etc.). Together
//! these two abstractions prevent incorrect use of indices across different
//! definition classes.
//!
//! # What this module is not
//!
//! However, this module cannot ensure that an index type is always used for the
//! [`IdxVec`] it originally came from. Imagine a scenario, in which multiple
//! Wasm module are used. Even though it would not make any sense to use indices
//! across multiple index spaces of the same definition class, detecting such
//! mis-use is out of the scope of this module.
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
