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

use crate::{
    core::reader::{types::FuncType, WasmReader},
    ValidationError,
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
    pub fn new(elements: Vec<T>) -> Result<Self, IdxVecOverflowError> {
        if u32::try_from(elements.len()).is_err() {
            return Err(IdxVecOverflowError);
        }

        Ok(Self {
            inner: elements.into_boxed_slice(),
            _phantom: PhantomData,
        })
    }

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

/// Index space for definitions that consist of imports and locals.
pub struct ExtendedIdxVec<I: Idx, T> {
    inner: Box<[T]>,
    num_imports: usize,
    _phantom: PhantomData<I>,
}

impl<I: Idx, T> Default for ExtendedIdxVec<I, T> {
    fn default() -> Self {
        Self {
            inner: Box::default(),
            num_imports: 0,
            _phantom: PhantomData,
        }
    }
}

impl<I: Idx, T: Clone> Clone for ExtendedIdxVec<I, T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            num_imports: self.num_imports,
            _phantom: PhantomData,
        }
    }
}

impl<I: Idx, T: core::fmt::Debug> core::fmt::Debug for ExtendedIdxVec<I, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(&*self.inner).finish()
    }
}

impl<I: Idx, T> ExtendedIdxVec<I, T> {
    /// Creates a new [`IdxVec`] with the given imported and local elements in
    /// it.
    ///
    /// If the number of total elements is larger than what can be addressed by
    /// a `u32`, i.e. `u32::MAX` elements, an error is returned instead.
    pub fn new(mut imports: Vec<T>, locals: Vec<T>) -> Result<Self, IdxVecOverflowError> {
        let imports_len = u32::try_from(imports.len()).map_err(|_| IdxVecOverflowError)?;
        let locals_len = u32::try_from(locals.len()).map_err(|_| IdxVecOverflowError)?;
        if imports_len.checked_add(locals_len).is_none() {
            return Err(IdxVecOverflowError);
        }

        let num_imports = imports.len();
        imports.extend(locals);

        Ok(Self {
            inner: imports.into_boxed_slice(),
            num_imports,
            _phantom: PhantomData,
        })
    }

    pub fn validate_index(&self, index: u32) -> Option<I> {
        let index_as_usize = usize::try_from(index).expect("architecture to be at least 32 bits");

        let _element = self.inner.get(index_as_usize)?;

        Some(I::new(index))
    }

    /// Gets an element from this vector by its index.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the index object was validated using the
    /// same vector as `self` or a different vector that was used to create
    /// `self` through [`ExtendedIdxVec::map`].
    pub unsafe fn get(&self, index: I) -> &T {
        let index =
            usize::try_from(index.into_inner()).expect("architecture to be at least 32 bits");

        // TODO use `unwrap_unchecked` when we are sure everything is sound and
        // our validation is properly tested
        self.inner
            .get(index)
            .expect("this to be a valid index due to the safety guarantees made by the caller")
    }

    /// Returns the length of this index space
    pub fn len(&self) -> u32 {
        u32::try_from(self.inner.len()).expect(
            "this to never be larger than u32::MAX because this was checked for in Self::new",
        )
    }

    /// Returns the length of the imported definitions part of this index space
    pub fn len_imported_definitions(&self) -> u32 {
        u32::try_from(self.num_imports).expect(
            "this to never be larger than u32::MAX, because this was checked for in Self::new",
        )
    }

    /// Returns the length of the locally-defined definitions part of this index
    /// space
    pub fn len_local_definitions(&self) -> u32 {
        self.len()
            .checked_sub(self.len_imported_definitions())
            .expect("that the number of imports is never larger than the total length of self")
    }

    /// Creates an equivalent index space for one that already exists while
    /// allowing elements to be mapped.
    ///
    /// Returns `None` if lengths do not match.
    // TODO maybe make this method take iterators instead of vectors
    pub fn map<R>(
        &self,
        new_imported_definitions: Vec<R>,
        new_local_definitions: Vec<R>,
    ) -> Option<ExtendedIdxVec<I, R>> {
        if u32::try_from(new_imported_definitions.len()).ok()? != self.len_imported_definitions()
            || u32::try_from(new_local_definitions.len()).ok()? != self.len_local_definitions()
        {
            return None;
        }

        let mut concatenated_definitions = new_imported_definitions;
        concatenated_definitions.extend(new_local_definitions);

        Some(ExtendedIdxVec {
            inner: concatenated_definitions.into_boxed_slice(),
            num_imports: self.num_imports,
            _phantom: PhantomData,
        })
    }

    pub fn iter(&self) -> core::slice::Iter<'_, T> {
        self.inner.iter()
    }

    pub fn iter_imported_definitions(&self) -> core::slice::Iter<'_, T> {
        self.inner
            .get(..self.num_imports)
            .expect("the imports length to never be larger than the total length")
            .iter()
    }

    pub fn iter_local_definitions(&self) -> core::slice::Iter<'_, T> {
        self.inner
            .get(self.num_imports..)
            .expect("the imports length to never be larger than the total length")
            .iter()
    }
}

/// A type index that is used to index into the types index space of some Wasm
/// module or module instance.
///
/// All Wasm indices, including this one, follow a type-state pattern. Refer to
/// [`indices`](crate::core::indices) for more information on this topic.
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

    /// Reads a type index from Wasm code without validating it. Using the
    /// returned type requires some other form of validation to be done.
    ///
    /// # Safety
    ///
    /// The caller must ensure that there is a valid type index in the
    /// [`WasmReader`].
    pub unsafe fn read_unchecked(wasm: &mut WasmReader) -> Self {
        let index = wasm.read_var_u32().unwrap();
        Self::new(index)
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

impl FuncIdx {
    /// Validates that a given index is a valid function index.
    ///
    /// On success a new [`FuncIdx`] is returned, otherwise a
    /// [`ValidationError`] is returned.
    pub fn validate<T>(
        index: u32,
        c_funcs: &ExtendedIdxVec<FuncIdx, T>,
    ) -> Result<Self, ValidationError> {
        c_funcs
            .validate_index(index)
            .ok_or(ValidationError::InvalidFuncIdx(index))
    }

    /// Reads a function index from Wasm code and validates that it is a valid
    /// index for a given functions vector.
    pub fn read_and_validate<T>(
        wasm: &mut WasmReader,
        c_funcs: &ExtendedIdxVec<FuncIdx, T>,
    ) -> Result<Self, ValidationError> {
        let index = wasm.read_var_u32()?;
        Self::validate(index, c_funcs)
    }

    /// Reads a function index from Wasm code without validating it.
    ///
    /// # Safety
    ///
    /// The caller must ensure that there is a valid function index in the
    /// [`WasmReader`] and that this index is valid for a specific [`IdxVec`]
    /// through [`Self::read_and_validate`].
    pub unsafe fn read_unchecked(wasm: &mut WasmReader) -> Self {
        let index = wasm.read_var_u32().unwrap();
        Self::new(index)
    }
}

pub type TableIdx = usize;
pub type MemIdx = usize;
pub type GlobalIdx = usize;
#[allow(dead_code)]
pub type ElemIdx = usize;
pub type DataIdx = usize;
pub type LocalIdx = usize;
pub type LabelIdx = usize;
