//! Methods to read WASM Types from a [WasmReader] object.
//!
//! See: <https://webassembly.github.io/spec/core/binary/types.html>

use alloc::vec::Vec;
use core::fmt::{Debug, Formatter};
use global::GlobalType;

use crate::core::reader::WasmReader;
use crate::core::utils::ToUsizeExt;
use crate::execution::assert_validated::UnwrapValidatedExt;
use crate::ValidationError;

pub mod data;
pub mod element;
pub mod export;
pub mod global;
pub mod import;
pub mod memarg;
pub mod opcode;
pub mod values;

/// <https://webassembly.github.io/spec/core/binary/types.html#number-types>
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NumType {
    I32,
    I64,
    F32,
    F64,
}

impl NumType {
    pub fn read(wasm: &mut WasmReader) -> Result<Self, ValidationError> {
        use NumType::*;

        let ty = match wasm.peek_u8()? {
            0x7F => I32,
            0x7E => I64,
            0x7D => F32,
            0x7C => F64,
            other => return Err(ValidationError::MalformedNumTypeDiscriminator(other)),
        };
        let _ = wasm.read_u8();

        Ok(ty)
    }
}

/// <https://webassembly.github.io/spec/core/binary/types.html#vector-types>
struct VecType;

impl VecType {
    fn read(wasm: &mut WasmReader) -> Result<Self, ValidationError> {
        match wasm.peek_u8()? {
            0x7b => {
                let _ = wasm.read_u8();
                Ok(VecType)
            }
            other => Err(ValidationError::MalformedVecTypeDiscriminator(other)),
        }
    }
}

/// <https://webassembly.github.io/spec/core/binary/types.html#reference-types>
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RefType {
    FuncRef,
    ExternRef,
}

impl RefType {
    pub fn read(wasm: &mut WasmReader) -> Result<RefType, ValidationError> {
        let ty = match wasm.peek_u8()? {
            0x70 => RefType::FuncRef,
            0x6F => RefType::ExternRef,
            other => return Err(ValidationError::MalformedRefTypeDiscriminator(other)),
        };
        let _ = wasm.read_u8();

        Ok(ty)
    }
}

/// <https://webassembly.github.io/spec/core/binary/types.html#reference-types>
/// TODO flatten [NumType] and [RefType] enums, as they are not used individually and `wasmparser` also does it.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[allow(clippy::all)]
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

impl ValType {
    pub fn read(wasm: &mut WasmReader) -> Result<Self, ValidationError> {
        if let Ok(numtype) = NumType::read(wasm).map(ValType::NumType) {
            return Ok(numtype);
        };
        if let Ok(vectype) = VecType::read(wasm).map(|_ty| ValType::VecType) {
            return Ok(vectype);
        };
        if let Ok(reftype) = RefType::read(wasm).map(ValType::RefType) {
            return Ok(reftype);
        }

        Err(ValidationError::MalformedValType)
    }
}

/// <https://webassembly.github.io/spec/core/binary/types.html#value-types>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResultType {
    pub valtypes: Vec<ValType>,
}

impl ResultType {
    pub fn read(wasm: &mut WasmReader) -> Result<Self, ValidationError> {
        let valtypes = wasm.read_vec(ValType::read)?;

        Ok(ResultType { valtypes })
    }
}

/// <https://webassembly.github.io/spec/core/binary/types.html#function-types>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuncType {
    pub params: ResultType,
    pub returns: ResultType,
}

impl FuncType {
    pub fn read(wasm: &mut WasmReader) -> Result<FuncType, ValidationError> {
        match wasm.read_u8()? {
            0x60 => {}
            other => return Err(ValidationError::MalformedFuncTypeDiscriminator(other)),
        };

        let params = ResultType::read(wasm)?;
        let returns = ResultType::read(wasm)?;

        Ok(FuncType { params, returns })
    }
}

/// <https://webassembly.github.io/spec/core/binary/instructions.html#binary-blocktype>
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockType {
    Empty,
    Returns(ValType),
    Type(u32),
}

impl BlockType {
    pub fn read(wasm: &mut WasmReader) -> Result<Self, ValidationError> {
        if wasm.peek_u8()? == 0x40 {
            // Empty block type
            let _ = wasm.read_u8().unwrap_validated();
            Ok(BlockType::Empty)
        } else if let Ok(val_ty) = wasm.handle_transaction(|wasm| ValType::read(wasm)) {
            // No parameters and given valtype as the result
            Ok(BlockType::Returns(val_ty))
        } else {
            // An index to a function type
            wasm.read_var_i33_as_u32().map(BlockType::Type)
        }
    }
}

impl BlockType {
    pub fn as_func_type(&self, func_types: &[FuncType]) -> Result<FuncType, ValidationError> {
        match self {
            BlockType::Empty => Ok(FuncType {
                params: ResultType {
                    valtypes: Vec::new(),
                },
                returns: ResultType {
                    valtypes: Vec::new(),
                },
            }),
            BlockType::Returns(val_type) => Ok(FuncType {
                params: ResultType {
                    valtypes: Vec::new(),
                },
                returns: ResultType {
                    valtypes: [*val_type].into(),
                },
            }),
            BlockType::Type(type_idx) => {
                let type_idx = type_idx.into_usize();
                func_types
                    .get(type_idx)
                    .cloned()
                    .ok_or(ValidationError::InvalidTypeIdx(type_idx))
            }
        }
    }
}

//https://webassembly.github.io/spec/core/valid/types.html#import-subtyping
pub trait ImportSubTypeRelation {
    // corresponds to "matches" (<=) in the spec
    fn is_subtype_of(&self, other: &Self) -> bool;
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

impl ImportSubTypeRelation for Limits {
    //https://webassembly.github.io/spec/core/valid/types.html#match-limits
    fn is_subtype_of(&self, other: &Self) -> bool {
        (self.min >= other.min)
            && (match other.max {
                None => true,
                Some(other_max) => match self.max {
                    None => false,
                    Some(self_max) => self_max <= other_max,
                },
            })
    }
}

impl Debug for Limits {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self.max {
            Some(max) => f.write_fmt(format_args!("{}..{}", self.min, max)),
            None => f.write_fmt(format_args!("{}..", self.min)),
        }
    }
}

impl Limits {
    pub fn read(wasm: &mut WasmReader) -> Result<Self, ValidationError> {
        let limits = match wasm.read_u8()? {
            0x00 => {
                let min = wasm.read_var_u32()?;
                Self { min, max: None }
            }
            0x01 => {
                let min = wasm.read_var_u32()?;
                let max = wasm.read_var_u32()?;
                Self {
                    min,
                    max: Some(max),
                }
            }
            other => return Err(ValidationError::MalformedLimitsDiscriminator(other)),
        };

        if let Some(max) = limits.max {
            if limits.min > max {
                return Err(ValidationError::MalformedLimitsMinLargerThanMax {
                    min: limits.min,
                    max,
                });
            }
        }

        Ok(limits)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TableType {
    pub et: RefType,
    pub lim: Limits,
}

// https://webassembly.github.io/spec/core/syntax/types.html#limits
impl TableType {
    pub fn read(wasm: &mut WasmReader) -> Result<Self, ValidationError> {
        let et = RefType::read(wasm)?;
        let mut lim = Limits::read(wasm)?;
        if lim.max.is_none() {
            lim.max = Some(u32::MAX)
        };
        trace!("Table: {:?}", Self { et, lim });
        Ok(Self { et, lim })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MemType {
    pub limits: Limits,
}

impl MemType {
    pub fn read(wasm: &mut WasmReader) -> Result<Self, ValidationError> {
        let limit = Limits::read(wasm)?;
        // Memory can only grow to 65536 pages of 64kb size (4GiB)
        if limit.min > (1 << 16) {
            return Err(ValidationError::MemoryTooLarge);
        }
        if let Some(max_limit) = limit.max {
            if max_limit > (1 << 16) {
                return Err(ValidationError::MemoryTooLarge);
            }
        }

        Ok(Self { limits: limit })
    }
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

impl ImportSubTypeRelation for ExternType {
    // https://webassembly.github.io/spec/core/valid/types.html#match-limits
    fn is_subtype_of(&self, other: &Self) -> bool {
        match self {
            ExternType::Table(self_table_type) => match other {
                ExternType::Table(other_table_type) => {
                    self_table_type.lim.is_subtype_of(&other_table_type.lim)
                        && self_table_type.et == other_table_type.et
                }
                _ => false,
            },
            ExternType::Mem(self_mem_type) => match other {
                ExternType::Mem(other_mem_type) => {
                    self_mem_type.limits.is_subtype_of(&other_mem_type.limits)
                }
                _ => false,
            },
            _ => self == other,
        }
    }
}
