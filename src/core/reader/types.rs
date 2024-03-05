//! Methods to read WASM Types from a [Wasm] object.
//!
//! See: <https://webassembly.github.io/spec/core/binary/types.html>

use alloc::vec::Vec;

use crate::core::reader::{WasmReadable, WasmReader};
use crate::Error;
use crate::Result;

/// https://webassembly.github.io/spec/core/binary/types.html#number-types
#[derive(Debug, Clone, PartialEq, Eq)]
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
}

/// https://webassembly.github.io/spec/core/binary/types.html#reference-types
#[derive(Debug, Clone, PartialEq, Eq)]
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
}

/// https://webassembly.github.io/spec/core/binary/types.html#reference-types
/// TODO flatten [NumType] and [RefType] enums, as they are not used individually and `wasmparser` also does it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValType {
    NumType(NumType),
    VecType,
    RefType(RefType),
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
}

/// https://webassembly.github.io/spec/core/binary/types.html#value-types
#[derive(Debug, Clone)]
pub struct ResultType {
    pub valtypes: Vec<ValType>,
}

impl WasmReadable for ResultType {
    fn read(wasm: &mut WasmReader) -> Result<Self> {
        let valtypes = wasm.read_vec(|wasm| ValType::read(wasm))?;

        Ok(ResultType { valtypes })
    }
}

/// https://webassembly.github.io/spec/core/binary/types.html#function-types
#[derive(Debug, Clone)]
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
}
