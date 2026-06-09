use alloc::vec::Vec;
use core::fmt;

use crate::core::structure::modules::indices::TypeIdx;

/// <https://webassembly.github.io/spec/core/binary/types.html#number-types>
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NumType {
    I32,
    I64,
    F32,
    F64,
}

/// <https://webassembly.github.io/spec/core/binary/types.html#vector-types>
pub struct VecType;

/// <https://webassembly.github.io/spec/core/binary/types.html#reference-types>
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RefType {
    FuncRef,
    ExternRef,
}

/// <https://webassembly.github.io/spec/core/binary/types.html#reference-types>
/// TODO flatten [NumType] and [RefType] enums, as they are not used individually and `wasmparser` also does it.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ValType {
    NumType(NumType),
    VecType,
    RefType(RefType),
}

impl ValType {
    pub const fn size(&self) -> usize {
        match self {
            Self::NumType(NumType::I32 | NumType::F32) => 4,
            Self::NumType(NumType::I64 | NumType::F64) => 8,
            Self::VecType => 16,
            Self::RefType(_) => todo!(),
        }
    }
}

/// <https://webassembly.github.io/spec/core/binary/types.html#value-types>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResultType {
    pub valtypes: Vec<ValType>,
}

/// <https://webassembly.github.io/spec/core/binary/types.html#function-types>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuncType {
    pub params: ResultType,
    pub returns: ResultType,
}

/// <https://webassembly.github.io/spec/core/binary/instructions.html#binary-blocktype>
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockType {
    Empty,
    Returns(ValType),
    Type(TypeIdx),
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Limits {
    pub min: u32,
    pub max: Option<u32>,
}

impl Limits {
    // since the maximum amount of bytes is u32::MAX, the page size is 1 << 16
    // the max no. of pages = max bytes / page size = u32::MAX / (1 << 16) = 1 << 16
    pub const MAX_MEM_PAGES: u32 = 1 << 16;
    // https://webassembly.github.io/reference-types/core/syntax/types.html#limits
    // memtype is defined in terms of limits, which go from 0 to u32::MAX
    pub const MAX_MEM_BYTES: u32 = u32::MAX;
    // https://webassembly.github.io/reference-types/core/exec/runtime.html#memory-instances
    // memory size is 65536 (1 << 16)
    pub const MEM_PAGE_SIZE: u32 = 1 << 16;
}

impl fmt::Debug for Limits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> core::fmt::Result {
        match self.max {
            Some(max) => f.write_fmt(format_args!("{}..{}", self.min, max)),
            None => f.write_fmt(format_args!("{}..", self.min)),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TableType {
    pub et: RefType,
    pub lim: Limits,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MemType {
    pub limits: Limits,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct GlobalType {
    pub ty: ValType,
    pub is_mut: bool,
}

// <https://webassembly.github.io/spec/core/valid/types.html#import-subtyping>
///<https://webassembly.github.io/spec/core/valid/types.html#external-types>
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ExternType {
    Func(FuncType),
    Table(TableType),
    Mem(MemType),
    Global(GlobalType),
}

#[derive(Debug)]
pub struct MemArg {
    pub offset: u32,
    pub align: u32,
}
