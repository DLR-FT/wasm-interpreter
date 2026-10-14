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
//! Additionally, the [`ExtendedIdxVec`] type is defined as to provide more
//! methods on an [`IdxVec`] that consists of imported and locally-defined
//! objects, i.e. functions, globals, tables and memories.
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

use alloc::boxed::Box;
use core::{convert::Infallible, marker::PhantomData, slice};

use crate::core::{
    trusted_exact_size_iterator::{
        IntoTrustedChainExt, TrustedExactSizeIterator, TryCollectIntoBoxedSliceExt,
    },
    utils::ToUsizeExt,
};

/// A trait for all index types.
///
/// This is used by [`IdxVec`] to create and read index types.
pub trait Idx: Copy + core::fmt::Debug + core::fmt::Display + Eq {
    fn new(index: u32) -> Self;

    fn into_inner(self) -> u32;
}

/// An immutable vector that can only be indexed by type-safe 32-bit indices.
///
/// Use [`IdxVec::new`] or [`IdxVec::default`] to create a new instance.
#[derive(Debug)]
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

/// An error for when an [`IdxVec`] is initialized with too many elements.
#[derive(Debug)]
pub struct IdxVecOverflowError;

impl<I: Idx, T> IdxVec<I, T> {
    /// Creates a new [`IdxVec`] with the given elements in it.
    ///
    /// If the number of elements is larger than what can be addressed by a
    /// `u32`, i.e. `u32::MAX`, an error is returned instead.
    pub fn new(elements: Box<[T]>) -> Result<Self, IdxVecOverflowError> {
        if u32::try_from(elements.len()).is_err() {
            return Err(IdxVecOverflowError);
        }

        Ok(Self {
            inner: elements,
            _phantom: PhantomData,
        })
    }

    /// Creates a new [`IdxVec`] from the given iterator's elements.
    ///
    /// If the number of elements is larger than what can be addressed by a
    /// `u32`, i.e. `u32::MAX`, an error is returned instead.
    #[inline]
    pub fn from_iter(
        elements: impl TrustedExactSizeIterator<Item = T>,
    ) -> Result<Self, IdxVecOverflowError> {
        // TODO: use into_ok instead of the hack for unwrapping the Result<_, Infallible> when the
        // never type is stabilized.
        Self::try_from_iter(elements.map(Ok::<T, Infallible>))
            .map(|result| result.unwrap_or_else(|infallible| match infallible {}))
    }

    /// Creates a new [`IdxVec`] from the given iterator's elements which can also be errors.
    ///
    /// If the number of elements is larger than what can be addressed by a
    /// `u32`, i.e. `u32::MAX`, an error is returned instead.
    #[inline]
    pub fn try_from_iter<E>(
        elements: impl TrustedExactSizeIterator<Item = Result<T, E>>,
    ) -> Result<Result<Self, E>, IdxVecOverflowError> {
        if u32::try_from(elements.len_trusted()).is_err() {
            return Err(IdxVecOverflowError.into());
        }

        Ok(elements.try_collect_into_boxed_slice().map(|inner| Self {
            inner,
            _phantom: PhantomData,
        }))
    }

    pub(crate) fn validate_index(&self, index: u32) -> Option<I> {
        let index_as_usize = index.into_usize();
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
    pub unsafe fn get(&self, index: I) -> &T {
        let index = index.into_inner().into_usize();

        debug_assert!(self.inner.get(index).is_some());
        // SAFETY: The caller ensures that the given index is valid in this vector. Because this
        // vector cannot shrink, the index must still be valid.
        unsafe { self.inner.get_unchecked(index) }
    }

    #[allow(unused)] // reason = "temporary until used by new index types"
    pub fn len(&self) -> u32 {
        u32::try_from(self.inner.len()).expect(
            "this to never be larger than u32::MAX, because this was checked for in Self::new",
        )
    }

    #[inline]
    pub fn iter(&self) -> slice::Iter<'_, T> {
        self.inner.iter()
    }

    #[inline]
    pub fn iter_enumerated(&self) -> impl Iterator<Item = (I, &T)> {
        self.inner.iter().enumerate().map(|(index, t)| {
            (
                I::new(
                    u32::try_from(index)
                        .expect("this vector to contain a maximum of 2^32-1 elements"),
                ),
                t,
            )
        })
    }
}

/// Index space for definitions that consist of imports and locals.
#[derive(Debug)]
pub struct ExtendedIdxVec<I: Idx, T> {
    inner: IdxVec<I, T>,
    num_imports: u32,
}

impl<I: Idx, T> Default for ExtendedIdxVec<I, T> {
    fn default() -> Self {
        Self {
            inner: IdxVec::default(),
            num_imports: 0,
        }
    }
}

impl<I: Idx, T: Clone> Clone for ExtendedIdxVec<I, T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            num_imports: self.num_imports,
        }
    }
}

impl<I: Idx, T> ExtendedIdxVec<I, T> {
    // # Safety
    //
    // `num_imports` must not be larger than `0..elements.len()`
    pub unsafe fn new(elements: Box<[T]>, num_imports: u32) -> Result<Self, IdxVecOverflowError> {
        debug_assert!(num_imports.into_usize() <= elements.len());
        Ok(Self {
            inner: IdxVec::new(elements)?,
            num_imports,
        })
    }
    /// Creates a new [`IdxVec`] with the given imported and local elements in
    /// it.
    ///
    /// If the number of total elements is larger than what can be addressed by
    /// a `u32`, i.e. `u32::MAX` elements, an error is returned instead.
    #[inline]
    pub fn from_iter(
        imports: impl TrustedExactSizeIterator<Item = T>,
        locals: impl TrustedExactSizeIterator<Item = T>,
    ) -> Result<Self, IdxVecOverflowError> {
        Self::try_from_iter(
            imports.map(Ok::<T, Infallible>),
            locals.map(Ok::<T, Infallible>),
        )
        .map(|result| result.unwrap_or_else(|infallible| match infallible {}))
    }

    /// Creates a new [`IdxVec`] with the given imported and local elements in
    /// it.
    ///
    /// If the number of total elements is larger than what can be addressed by
    /// a `u32`, i.e. `u32::MAX` elements, an error is returned instead.
    #[inline]
    pub fn try_from_iter<E>(
        imports: impl TrustedExactSizeIterator<Item = Result<T, E>>,
        locals: impl TrustedExactSizeIterator<Item = Result<T, E>>,
    ) -> Result<Result<Self, E>, IdxVecOverflowError> {
        let imports_len = u32::try_from(imports.len_trusted()).map_err(|_| IdxVecOverflowError)?;
        let locals_len = u32::try_from(locals.len_trusted()).map_err(|_| IdxVecOverflowError)?;

        if imports_len.checked_add(locals_len).is_none() {
            return Err(IdxVecOverflowError);
        }

        // Chaining two `TrustedExactSizeIterator`s can be cumbersome.
        let chained_untrusted = imports.chain(locals);
        // SAFETY: The sum of both iterator's lengths is `num_total` which did not overflow.
        // Therefore, chaining both iterators does not overflow the upper size bound.
        let chained = unsafe { chained_untrusted.try_into_trusted().unwrap_unchecked() };

        let maybe_inner = IdxVec::try_from_iter(chained)?;

        Ok(maybe_inner.map(|inner| Self {
            inner,
            num_imports: imports_len,
        }))
    }

    pub fn len_imported_definitions(&self) -> u32 {
        self.num_imports
    }

    /// Returns the length of the locally-defined definitions part of this index
    /// space
    pub fn len_local_definitions(&self) -> u32 {
        self.inner
            .len()
            .checked_sub(self.num_imports)
            .expect("that the number of imports is never larger than the total length of self")
    }

    pub fn imported_definitions(&self) -> &[T] {
        self.inner
            .inner
            .get(..self.num_imports.into_usize())
            .expect("the imports length to never be larger than the total length")
    }

    pub fn local_definitions(&self) -> &[T] {
        self.inner
            .inner
            .get(self.num_imports.into_usize()..)
            .expect("the imports length to never be larger than the total length")
    }

    pub fn iter_imported_definitions(&self) -> core::slice::Iter<'_, T> {
        self.imported_definitions().iter()
    }

    pub fn iter_local_definitions(&self) -> core::slice::Iter<'_, T> {
        self.local_definitions().iter()
    }

    pub fn inner(&self) -> &IdxVec<I, T> {
        &self.inner
    }

    pub fn into_inner(self) -> IdxVec<I, T> {
        self.inner
    }
}

/// A type index that is used to index into the types index space of some Wasm
/// module or module instance.
///
/// All Wasm indices, including this one, follow a type-state pattern. Refer to
/// [`indices`](crate::core::structure::modules::indices) for more information on this topic.
///
/// See: WebAssembly Specification 2.0 - 2.5.1 - Indices
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TypeIdx(u32);

impl core::fmt::Display for TypeIdx {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "type index {}", self.0)
    }
}

impl Idx for TypeIdx {
    fn new(index: u32) -> Self {
        Self(index)
    }

    fn into_inner(self) -> u32 {
        self.0
    }
}

impl TypeIdx {
    /// Creates a new type index directly from some index.
    ///
    /// Note: This constructor is only available for type indices, since these
    /// are the only indices that can be encoded using special 33-bit integers.
    pub fn new(index: u32) -> Self {
        Self(index)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FuncIdx(u32);

impl core::fmt::Display for FuncIdx {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "function index {}", self.0)
    }
}

impl Idx for FuncIdx {
    fn new(index: u32) -> Self {
        Self(index)
    }

    fn into_inner(self) -> u32 {
        self.0
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TableIdx(u32);

impl core::fmt::Display for TableIdx {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "table index {}", self.0)
    }
}

impl Idx for TableIdx {
    fn new(index: u32) -> Self {
        Self(index)
    }

    fn into_inner(self) -> u32 {
        self.0
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemIdx(u32);

impl core::fmt::Display for MemIdx {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "memory index {}", self.0)
    }
}

impl Idx for MemIdx {
    fn new(index: u32) -> Self {
        Self(index)
    }

    fn into_inner(self) -> u32 {
        self.0
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct GlobalIdx(u32);

impl core::fmt::Display for GlobalIdx {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "global index {}", self.0)
    }
}

impl Idx for GlobalIdx {
    fn new(index: u32) -> Self {
        Self(index)
    }

    fn into_inner(self) -> u32 {
        self.0
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ElemIdx(u32);

impl core::fmt::Display for ElemIdx {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "element index {}", self.0)
    }
}

impl Idx for ElemIdx {
    fn new(index: u32) -> Self {
        Self(index)
    }

    fn into_inner(self) -> u32 {
        self.0
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DataIdx(u32);

impl core::fmt::Display for DataIdx {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "data index {}", self.0)
    }
}

impl Idx for DataIdx {
    fn new(index: u32) -> Self {
        Self(index)
    }

    fn into_inner(self) -> u32 {
        self.0
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LocalIdx(pub(crate) u32);

impl core::fmt::Display for LocalIdx {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "local index {}", self.0)
    }
}

impl LocalIdx {
    pub fn into_inner(self) -> u32 {
        self.0
    }
}
