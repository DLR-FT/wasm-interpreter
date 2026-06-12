//! # Valid Type Definitions
//!
//! This module defines Rust structs for Wasm types as defined in [^structure-types], which haveh
//! already been validated according to [^valid-types]. Some structs may always be valid, while
//! other (e.g. [`Limits`]) have invariants associated with them which are not expressed through
//! Wasm's type system.
//!
//! ## Types from the Instructions Chapter
//!
//! The Wasm specification defines most of its types in the types chapter[^structure-types].
//! However, block types and memargs are defined in the instructions
//! chapter[^structure-instructions] alongside the instructions which use them. For completeness
//! these types are included in this Rust module as [`BlockType`] and [`MemArg`].
//!
//! [^structure-types]: [WebAssembly Specification 2.0 - 2.3. Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#types%E2%91%A0).
//! [^valid-types]: [WebAssembly Specification 2.0 - 3.2. Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#types%E2%91%A4).
//! [^structure-instructions]: [WebAssembly Specification 2.0 - 2.4. Instructions](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#instructions%E2%91%A0).

use alloc::{vec, vec::Vec};
use core::fmt;

use crate::core::structure::modules::indices::TypeIdx;

/// A number type
///
/// See: [WebAssembly Specification 2.0 - 2.3.1. Number Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#syntax-numtype).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NumType {
    I32,
    I64,
    F32,
    F64,
}

/// A vector type
///
/// See: [WebAssembly Specification 2.0 - 2.3.2. Vector Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#syntax-vectype).
pub struct VecType;

/// A reference type
///
/// See: [WebAssembly Specification 2.0 - 2.3.3. Reference Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#syntax-reftype).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RefType {
    FuncRef,
    ExternRef,
}

/// A value type
///
/// See: [WebAssembly Specification 2.0 - 2.3.4. Value Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#syntax-valtype).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ValType {
    NumType(NumType),
    VecType,
    RefType(RefType),
}

/// A result type
///
/// See: [WebAssembly Specification 2.0 - 2.3.5. Result Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#syntax-resulttype).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResultType {
    pub valtypes: Vec<ValType>,
}

/// A function type
///
/// See: [WebAssembly Specification 2.0 - 2.3.6. Function Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#syntax-functype).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuncType {
    pub params: ResultType,
    pub returns: ResultType,
}

impl FuncType {
    pub fn new_empty() -> Self {
        Self {
            params: ResultType::default(),
            returns: ResultType::default(),
        }
    }

    pub fn new_returning(single_return_value: ValType) -> Self {
        Self {
            params: ResultType::default(),
            returns: ResultType {
                valtypes: vec![single_return_value],
            },
        }
    }

    pub fn is_empty(&self) -> bool {
        self.params.valtypes.is_empty() && self.returns.valtypes.is_empty()
    }
}

/// A block type
///
/// See: [WebAssembly Specification 2.0 - 2.4.8. Control Instructions](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#syntax-blocktype).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockType {
    // TODO Combine `Empty` and `Returns` into `MaybeReturns(Option<ValType>)`
    Empty,
    Returns(ValType),
    Type(TypeIdx),
}

/// A limits object[^spec]
///
/// This object is only valid within a certain range `k`[^validation] that depends on the context in
/// which these limits exist. This can be a [`MemType`] or a [`TableType`].
///
/// [^spec]: [WebAssembly Specification 2.0 - 2.3.7. Limits](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#syntax-limits).
/// [^validation]: [WebAssembly Specification 2.0 - 3.2.1. Limits](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#limits%E2%91%A2).
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Limits {
    pub min: u32,
    /// If this is `Some(n)`, `n` must be greater or equal to `self.min`
    pub max: Option<u32>,
}

impl fmt::Debug for Limits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> core::fmt::Result {
        match self.max {
            Some(max) => f.write_fmt(format_args!("{}..{}", self.min, max)),
            None => f.write_fmt(format_args!("{}..", self.min)),
        }
    }
}

/// A table type
///
/// See: [WebAssembly Specification 2.0 - 2.3.9. Table Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#syntax-tabletype).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TableType {
    pub et: RefType,
    /// These limits must always be valid within range 2^32 - 1.
    ///
    /// See: [WebAssembly Specification 2.0 - 3.2.4. Table Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#table-types%E2%91%A2).
    pub lim: Limits,
}

/// A memory type
///
/// See: [WebAssembly Specification 2.0 - 2.3.8. Memory Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#syntax-memtype).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MemType {
    /// These limits must always be valid within range 2^16.
    ///
    /// See: [WebAssembly Specification 2.0 - 3.2.5. Memory Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#memory-types%E2%91%A2).
    pub limits: Limits,
}

/// A global type
///
/// See: [WebAssembly Specification 2.0 - 2.3.10](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#syntax-globaltype).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct GlobalType {
    pub ty: ValType,
    pub is_mut: bool,
}

/// An external type
///
/// This type exists because cloning [`FuncType`]s is expensive and often unnecessary. Use
/// [`ExternType`] if cloning is wanted.
///
/// See: [WebAssembly Specification 2.0 - 2.3.11. External Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#syntax-externtype).
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ExternTypeRef<'a> {
    Func(&'a FuncType),
    Table(TableType),
    Mem(MemType),
    Global(GlobalType),
}

/// An owned external type
///
/// See: [WebAssembly Specification 2.0 - 2.3.11. External Types](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#syntax-externtype).
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ExternType {
    Func(FuncType),
    Table(TableType),
    Mem(MemType),
    Global(GlobalType),
}

impl ExternType {
    pub fn as_ref(&self) -> ExternTypeRef<'_> {
        match self {
            ExternType::Func(func_type) => ExternTypeRef::Func(func_type),
            ExternType::Table(table_type) => ExternTypeRef::Table(*table_type),
            ExternType::Mem(mem_type) => ExternTypeRef::Mem(*mem_type),
            ExternType::Global(global_type) => ExternTypeRef::Global(*global_type),
        }
    }
}

impl ExternTypeRef<'_> {
    pub fn to_owned(self) -> ExternType {
        match self {
            ExternTypeRef::Func(func_type) => ExternType::Func(func_type.clone()),
            ExternTypeRef::Table(table_type) => ExternType::Table(table_type),
            ExternTypeRef::Mem(mem_type) => ExternType::Mem(mem_type),
            ExternTypeRef::Global(global_type) => ExternType::Global(global_type),
        }
    }
}

/// A memarg
///
/// See: [WebAssembly Specification 2.0 - 2.4.7. Memory Instructions](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#syntax-memarg).
#[derive(Debug)]
pub struct MemArg {
    pub offset: u32,
    pub align: u32,
}
