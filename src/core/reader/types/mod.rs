//! Methods to read WASM Types from a [WasmReader] object.
//!
//! See: <https://webassembly.github.io/spec/core/binary/types.html>

use alloc::vec::Vec;
use core::fmt::{Debug, Formatter};

use crate::core::reader::{WasmReadable, WasmReader};
use crate::execution::assert_validated::UnwrapValidatedExt;
use crate::value::{ExternAddr, FuncAddr, Ref};
use crate::Result;
use crate::{unreachable_validated, Error};

pub mod data;
pub mod element;
pub mod export;
pub mod function_code_header;
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

impl WasmReadable for NumType {
    fn read(wasm: &mut WasmReader) -> Result<Self> {
        use NumType::*;

        let ty = match wasm.peek_u8()? {
            0x7F => I32,
            0x7E => I64,
            0x7D => F32,
            0x7C => F64,
            _ => return Err(Error::InvalidNumType),
        };
        let _ = wasm.read_u8();

        Ok(ty)
    }

    fn read_unvalidated(wasm: &mut WasmReader) -> Self {
        use NumType::*;
        match wasm.read_u8().unwrap_validated() {
            0x7F => I32,
            0x7E => I64,
            0x7D => F32,
            0x7C => F64,
            _ => unreachable_validated!(),
        }
    }
}

/// <https://webassembly.github.io/spec/core/binary/types.html#vector-types>
struct VecType;

impl WasmReadable for VecType {
    fn read(wasm: &mut WasmReader) -> Result<Self> {
        let 0x7B = wasm.peek_u8()? else {
            return Err(Error::InvalidVecType);
        };
        let _ = wasm.read_u8();

        Ok(VecType)
    }

    fn read_unvalidated(wasm: &mut WasmReader) -> Self {
        let 0x7B = wasm.read_u8().unwrap_validated() else {
            unreachable_validated!()
        };

        VecType
    }
}

/// <https://webassembly.github.io/spec/core/binary/types.html#reference-types>
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RefType {
    FuncRef,
    ExternRef,
}

impl RefType {
    /// TODO: we have to make sure they are NOT null Refs, but still, they are
    /// not valid ones as we cast them from RefTypes which don't hold addresses
    /// per-se
    pub fn to_null_ref(&self) -> Ref {
        match self {
            RefType::ExternRef => Ref::Extern(ExternAddr::null()),
            RefType::FuncRef => Ref::Func(FuncAddr::null()),
        }
    }
}

impl RefType {
    pub fn from_byte(byte: u8) -> Result<RefType> {
        match byte {
            0x70 => Ok(RefType::FuncRef),
            0x6F => Ok(RefType::ExternRef),
            _ => Err(Error::InvalidRefType),
        }
    }
}

impl WasmReadable for RefType {
    fn read(wasm: &mut WasmReader) -> Result<RefType> {
        let ty = match wasm.peek_u8()? {
            0x70 => RefType::FuncRef,
            0x6F => RefType::ExternRef,
            _ => return Err(Error::InvalidRefType),
        };
        let _ = wasm.read_u8();

        Ok(ty)
    }

    fn read_unvalidated(wasm: &mut WasmReader) -> Self {
        match wasm.read_u8().unwrap_validated() {
            0x70 => RefType::FuncRef,
            0x6F => RefType::ExternRef,
            _ => unreachable_validated!(),
        }
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

impl WasmReadable for ValType {
    fn read(wasm: &mut WasmReader) -> Result<Self> {
        if let Ok(numtype) = NumType::read(wasm).map(ValType::NumType) {
            return Ok(numtype);
        };
        if let Ok(vectype) = VecType::read(wasm).map(|_ty| ValType::VecType) {
            return Ok(vectype);
        };
        if let Ok(reftype) = RefType::read(wasm).map(ValType::RefType) {
            return Ok(reftype);
        }

        Err(Error::InvalidValType)
    }

    fn read_unvalidated(wasm: &mut WasmReader) -> Self {
        if let Ok(numtype) = NumType::read(wasm).map(ValType::NumType) {
            return numtype;
        };
        if let Ok(vectype) = VecType::read(wasm).map(|_ty| ValType::VecType) {
            return vectype;
        };
        if let Ok(reftype) = RefType::read(wasm).map(ValType::RefType) {
            return reftype;
        }

        unreachable!()
    }
}

/// <https://webassembly.github.io/spec/core/binary/types.html#value-types>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResultType {
    pub valtypes: Vec<ValType>,
}

impl WasmReadable for ResultType {
    fn read(wasm: &mut WasmReader) -> Result<Self> {
        let valtypes = wasm.read_vec(ValType::read)?;

        Ok(ResultType { valtypes })
    }

    fn read_unvalidated(wasm: &mut WasmReader) -> Self {
        let valtypes = wasm
            .read_vec(|wasm| Ok(ValType::read_unvalidated(wasm)))
            .unwrap_validated();

        ResultType { valtypes }
    }
}

/// <https://webassembly.github.io/spec/core/binary/types.html#function-types>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuncType {
    pub params: ResultType,
    pub returns: ResultType,
}

impl WasmReadable for FuncType {
    fn read(wasm: &mut WasmReader) -> Result<FuncType> {
        let 0x60 = wasm.read_u8()? else {
            return Err(Error::InvalidFuncType);
        };

        let params = ResultType::read(wasm)?;
        let returns = ResultType::read(wasm)?;

        Ok(FuncType { params, returns })
    }

    fn read_unvalidated(wasm: &mut WasmReader) -> Self {
        let 0x60 = wasm.read_u8().unwrap_validated() else {
            unreachable_validated!()
        };

        let params = ResultType::read_unvalidated(wasm);
        let returns = ResultType::read_unvalidated(wasm);

        FuncType { params, returns }
    }
}

/// <https://webassembly.github.io/spec/core/binary/instructions.html#binary-blocktype>
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockType {
    Empty,
    Returns(ValType),
    Type(u32),
}

impl WasmReadable for BlockType {
    fn read(wasm: &mut WasmReader) -> Result<Self> {
        if wasm.peek_u8()? as i8 == 0x40 {
            // Empty block type
            let _ = wasm.read_u8().unwrap_validated();
            Ok(BlockType::Empty)
        } else if let Ok(val_ty) = wasm.handle_transaction(|wasm| ValType::read(wasm)) {
            // No parameters and given valtype as the result
            Ok(BlockType::Returns(val_ty))
        } else {
            // An index to a function type
            wasm.read_var_i33()
                .and_then(|idx| idx.try_into().map_err(|_| Error::InvalidFuncTypeIdx))
                .map(BlockType::Type)
        }
    }

    fn read_unvalidated(wasm: &mut WasmReader) -> Self {
        if wasm.peek_u8().unwrap_validated() as i8 == 0x40 {
            // Empty block type
            let _ = wasm.read_u8();

            BlockType::Empty
        } else if let Ok(val_ty) = wasm.handle_transaction(|wasm| ValType::read(wasm)) {
            // No parameters and given valtype as the result
            BlockType::Returns(val_ty)
        } else {
            // An index to a function type
            BlockType::Type(
                wasm.read_var_i33()
                    .unwrap_validated()
                    .try_into()
                    .unwrap_validated(),
            )
        }
    }
}

impl BlockType {
    pub fn as_func_type(&self, func_types: &[FuncType]) -> Result<FuncType> {
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
                let type_idx: usize = (*type_idx)
                    .try_into()
                    .map_err(|_| Error::InvalidFuncTypeIdx)?;

                func_types
                    .get(type_idx)
                    .cloned()
                    .ok_or(Error::InvalidFuncTypeIdx)
            }
        }
    }
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

impl Debug for Limits {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self.max {
            Some(max) => f.write_fmt(format_args!("{}..{}", self.min, max)),
            None => f.write_fmt(format_args!("{}..", self.min)),
        }
    }
}

impl WasmReadable for Limits {
    fn read(wasm: &mut WasmReader) -> Result<Self> {
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
            other => return Err(Error::InvalidLimitsType(other)),
        };

        if let Some(max) = limits.max {
            if limits.min > max {
                return Err(Error::InvalidLimit);
            }
        }

        Ok(limits)
    }

    fn read_unvalidated(wasm: &mut WasmReader) -> Self {
        match wasm.read_u8().unwrap_validated() {
            0x00 => {
                let min = wasm.read_var_u32().unwrap_validated();
                Self { min, max: None }
            }
            0x01 => {
                let min = wasm.read_var_u32().unwrap_validated();
                let max = wasm.read_var_u32().unwrap_validated();
                Self {
                    min,
                    max: Some(max),
                }
            }
            _ => unreachable_validated!(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TableType {
    pub et: RefType,
    pub lim: Limits,
}

// https://webassembly.github.io/spec/core/syntax/types.html#limits
impl WasmReadable for TableType {
    fn read(wasm: &mut WasmReader) -> Result<Self> {
        let et = RefType::read(wasm)?;
        let mut lim = Limits::read(wasm)?;
        if lim.max.is_none() {
            lim.max = Some(u32::MAX)
        };
        let table_type = Self { et, lim };
        trace!("Table: {:?}", table_type);
        Ok(Self { et, lim })
    }

    fn read_unvalidated(wasm: &mut WasmReader) -> Self {
        let et = RefType::read_unvalidated(wasm);
        let mut lim = Limits::read_unvalidated(wasm);
        if lim.max.is_none() {
            lim.max = Some(u32::MAX)
        };
        Self { et, lim }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MemType {
    pub limits: Limits,
}

impl WasmReadable for MemType {
    fn read(wasm: &mut WasmReader) -> Result<Self> {
        let mut limit = Limits::read(wasm)?;
        // Memory can only grow to 65536 pages of 64kb size (4GiB)
        if limit.min > (1 << 16) {
            return Err(Error::MemSizeTooBig);
        }
        if limit.max.is_none() {
            limit.max = Some(1 << 16);
        } else if limit.max.is_some() {
            let max_limit = limit.max.unwrap();
            if max_limit > (1 << 16) {
                return Err(Error::MemSizeTooBig);
            }
        }
        Ok(Self { limits: limit })
    }

    fn read_unvalidated(wasm: &mut WasmReader) -> Self {
        Self {
            limits: Limits::read_unvalidated(wasm),
        }
    }
}
