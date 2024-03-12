//! Methods to read WASM Types from a [Wasm] object.
//!
//! See: <https://webassembly.github.io/spec/core/binary/types.html>

use alloc::vec::Vec;
use core::fmt::{Debug, Formatter, Write};

use crate::core::reader::{WasmReadable, WasmReader};
use crate::execution::assert_validated::UnwrapValidatedExt;
use crate::Result;
use crate::{unreachable_validated, Error};

pub mod export;
pub mod function_code_header;
pub mod global;
pub mod import;
pub mod memarg;
pub mod values;

/// https://webassembly.github.io/spec/core/binary/types.html#number-types
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

/// https://webassembly.github.io/spec/core/binary/types.html#vector-types
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

/// https://webassembly.github.io/spec/core/binary/types.html#reference-types
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RefType {
    FuncRef,
    ExternRef,
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

/// https://webassembly.github.io/spec/core/binary/types.html#reference-types
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

impl WasmReadable for ValType {
    fn read(wasm: &mut WasmReader) -> Result<Self> {
        let numtype = NumType::read(wasm).map(|ty| ValType::NumType(ty));
        let vectype = VecType::read(wasm).map(|_ty| ValType::VecType);
        let reftype = RefType::read(wasm).map(|ty| ValType::RefType(ty));

        numtype
            .or(vectype)
            .or(reftype)
            .map_err(|_| Error::InvalidValType)
    }

    fn read_unvalidated(wasm: &mut WasmReader) -> Self {
        let numtype = NumType::read(wasm).map(|ty| ValType::NumType(ty));
        let vectype = VecType::read(wasm).map(|_ty| ValType::VecType);
        let reftype = RefType::read(wasm).map(|ty| ValType::RefType(ty));

        numtype.or(vectype).or(reftype).unwrap_validated()
    }
}

/// https://webassembly.github.io/spec/core/binary/types.html#value-types
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResultType {
    pub valtypes: Vec<ValType>,
}

impl WasmReadable for ResultType {
    fn read(wasm: &mut WasmReader) -> Result<Self> {
        let valtypes = wasm.read_vec(|wasm| ValType::read(wasm))?;

        Ok(ResultType { valtypes })
    }

    fn read_unvalidated(wasm: &mut WasmReader) -> Self {
        let valtypes = wasm
            .read_vec(|wasm| Ok(ValType::read_unvalidated(wasm)))
            .unwrap_validated();

        ResultType { valtypes }
    }
}

/// https://webassembly.github.io/spec/core/binary/types.html#function-types
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

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Limits {
    pub min: u32,
    pub max: Option<u32>,
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

impl WasmReadable for TableType {
    fn read(wasm: &mut WasmReader) -> Result<Self> {
        let et = RefType::read(wasm)?;
        let lim = Limits::read(wasm)?;
        Ok(Self { et, lim })
    }

    fn read_unvalidated(wasm: &mut WasmReader) -> Self {
        let et = RefType::read_unvalidated(wasm);
        let lim = Limits::read_unvalidated(wasm);

        Self { et, lim }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MemType {
    pub limits: Limits,
}

impl WasmReadable for MemType {
    fn read(wasm: &mut WasmReader) -> Result<Self> {
        Ok(Self {
            limits: Limits::read(wasm)?,
        })
    }

    fn read_unvalidated(wasm: &mut WasmReader) -> Self {
        Self {
            limits: Limits::read_unvalidated(wasm),
        }
    }
}
