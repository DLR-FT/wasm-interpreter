// /// This macro defines index types. Currently (2024-06-10) all indices are [`u32`].
// /// See <https://webassembly.github.io/spec/core/binary/modules.html#indices> for more information.
// macro_rules! def_idx_types {
//     ($($name:ident),*) => {
//         $(
//             /// <https://webassembly.github.io/spec/core/binary/modules.html#indices>
//             pub type $name = usize;
//         )*
//     };
// }

// // #[allow(dead_code)]
// def_idx_types!(TypeIdx, FuncIdx, TableIdx, MemIdx, GlobalIdx, /* ElemIdx, DataIdx, */ LocalIdx/* , LabelIdx */);

// TODO check whether is is clever to internally use usize instead of u32; potential problems are:
// - unsound on architectures where `usize` < `u32`
// - wasteful in memory on architectures where `usize` > `u32`

use core::marker::PhantomData;

use alloc::{boxed::Box, vec::Vec};

use crate::{
    core::reader::{types::FuncType, WasmReader},
    ValidationError,
};

/// A trait for all index types.
///
/// This is used by [`IdxVec`] to create and read index types.
pub(crate) trait Idx: Copy + core::fmt::Debug + core::fmt::Display + Eq {
    /// # Safety
    ///
    /// The caller must ensure that the given index is valid for some
    /// [`IdxVec`], so it can later be used to access the element it points to
    /// in that vector.
    unsafe fn new_unchecked(index: u32) -> Self;

    fn into_inner(self) -> u32;
}

/// An immutable vector that can only be indexed by type-safe indices.
///
/// Use [`IdxVec::new`] or [`IdxVec::default`] to create a new instance.
pub(crate) struct IdxVec<I: Idx, T> {
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
        f.debug_list().entries(&self.inner).finish()
    }
}

impl<I: Idx, T> IdxVec<I, T> {
    pub fn new(elements: Vec<T>) -> Self {
        Self {
            inner: elements.into_boxed_slice(),
            _phantom: PhantomData,
        }
    }

    pub fn validate_index(&self, index: u32) -> Option<I> {
        let index_as_usize = usize::try_from(index).expect("architecture to be at least 32 bits");

        let _element = self.inner.get(index_as_usize)?;

        // SAFETY: It was just validated that this index is valid for the
        // current vector.
        Some(unsafe { I::new_unchecked(index) })
    }

    /// Gets an element from this vector by its index.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the index object was validated using the
    /// same vector as `self`.
    pub unsafe fn get(&self, index: I) -> &T {
        let index =
            usize::try_from(index.into_inner()).expect("architecture to be at least 32 bits");

        self.inner.get(index).expect("this to be a valid index") // TODO use `unwrap_unchecked` instead
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TypeIdx(u32);

impl core::fmt::Display for TypeIdx {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "type index {}", self.0)
    }
}

impl Idx for TypeIdx {
    unsafe fn new_unchecked(index: u32) -> Self {
        Self(index)
    }

    fn into_inner(self) -> u32 {
        self.0
    }
}

impl TypeIdx {
    /// Validates that a given index is a valid type index.
    ///
    /// On success a new [`TypeIdx`] is returned, otherwise a
    /// [`ValidationError`] is returned.
    pub fn validate(
        index: u32,
        c_types: &IdxVec<TypeIdx, FuncType>,
    ) -> Result<Self, ValidationError> {
        c_types
            .validate_index(index)
            .ok_or(ValidationError::InvalidTypeIdx(index))
    }

    /// Reads a type index from Wasm code and validates that it is a valid index
    /// for a given types vector.
    pub fn read_and_validate(
        wasm: &mut WasmReader,
        c_types: &IdxVec<TypeIdx, FuncType>,
    ) -> Result<Self, ValidationError> {
        let index = wasm.read_var_u32()?;
        Self::validate(index, c_types)
    }

    /// Reads a type index from Wasm code without validating it.
    ///
    /// # Safety
    ///
    /// The caller must ensure that there is a valid type index in the
    /// [`WasmReader`] and that this index is valid for a specific [`IdxVec`].
    pub unsafe fn read_unchecked(wasm: &mut WasmReader) -> Self {
        let index = wasm.read_var_u32().unwrap();

        // SAFETY: The caller guarantees that the index that was just read is
        // valid.
        unsafe { Self::new_unchecked(index) }
    }
}

pub type FuncIdx = usize;
pub type TableIdx = usize;
pub type MemIdx = usize;
pub type GlobalIdx = usize;
#[allow(dead_code)]
pub type ElemIdx = usize;
pub type DataIdx = usize;
pub type LocalIdx = usize;
#[allow(dead_code)]
pub type LabelIdx = usize;
