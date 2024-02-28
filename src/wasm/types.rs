//! Methods to read WASM Types from a [Wasm] object.
//!
//! See: <https://webassembly.github.io/spec/core/binary/types.html>

use alloc::vec::Vec;

use crate::wasm::Wasm;
use crate::Error;
use crate::Result;

/// https://webassembly.github.io/spec/core/binary/types.html#number-types
#[derive(Debug, Clone)]
pub enum NumType {
    I32,
    I64,
    F32,
    F64,
}
impl<'a> Wasm<'a> {
    pub fn read_numtype(&mut self) -> Result<NumType> {
        use NumType::*;

        let ty = match self.peek_byte()? {
            0x7F => I32,
            0x7E => I64,
            0x7D => F32,
            0x7C => F64,
            _ => return Err(Error::InvalidNumType),
        };

        let _ = self.read_u8();

        Ok(ty)
    }
}

// https://webassembly.github.io/spec/core/binary/types.html#vector-types
impl<'a> Wasm<'a> {
    pub fn read_vectype(&mut self) -> Result<()> {
        let 0x7B = self.peek_byte()? else {
            return Err(Error::InvalidVecType);
        };
        let _ = self.read_u8();

        Ok(())
    }
}

/// https://webassembly.github.io/spec/core/binary/types.html#reference-types
#[derive(Debug, Clone)]
pub enum RefType {
    FuncRef,
    ExternRef,
}

impl<'a> Wasm<'a> {
    pub fn read_reftype(&mut self) -> Result<RefType> {
        let ty = match self.peek_byte()? {
            0x70 => RefType::FuncRef,
            0x6F => RefType::ExternRef,
            _ => return Err(Error::InvalidRefType),
        };
        let _ = self.read_u8();
        Ok(ty)
    }
}

/// https://webassembly.github.io/spec/core/binary/types.html#reference-types
#[derive(Debug, Clone)]
pub enum ValType {
    NumType(NumType),
    VecType,
    RefType(RefType),
}
impl<'a> Wasm<'a> {
    pub fn read_valtype(&mut self) -> Result<ValType> {
        let numtype = self.read_numtype().map(|t| ValType::NumType(t));
        let vectype = self.read_vectype().map(|_t| ValType::VecType);
        let reftype = self.read_reftype().map(|t| ValType::RefType(t));

        numtype
            .or(vectype)
            .or(reftype)
            .map_err(|_| Error::InvalidValType)
    }
}

/// https://webassembly.github.io/spec/core/binary/types.html#value-types
#[derive(Debug, Clone)]
pub struct ResultType {
    valtypes: Vec<ValType>,
}

impl<'a> Wasm<'a> {
    pub fn read_resulttype(&mut self) -> Result<ResultType> {
        let valtypes = self.read_vec(|wasm| wasm.read_valtype())?;

        Ok(ResultType { valtypes })
    }
}

/// https://webassembly.github.io/spec/core/binary/types.html#function-types
#[derive(Debug, Clone)]
pub struct FuncType {
    params: ResultType,
    returns: ResultType,
}

impl<'a> Wasm<'a> {
    pub fn read_functype(&mut self) -> Result<FuncType> {
        let 0x60 = self.read_u8()? else {
            return Err(Error::InvalidFuncType);
        };

        let params = self.read_resulttype()?;
        let returns = self.read_resulttype()?;

        Ok(FuncType { params, returns })
    }
}
