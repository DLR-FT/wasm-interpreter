use core::str::Utf8Error;

use crate::section::SectionTy;
use crate::wasm::types::ValType;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Error {
    InvalidMagic,
    InvalidVersion,
    MalformedUtf8String(Utf8Error),
    MissingValue,
    InvalidSectionType(u8),
    SectionOutOfOrder(SectionTy),
    InvalidNumType,
    InvalidVecType,
    InvalidFuncType,
    InvalidRefType,
    InvalidValType,
    InvalidExportDesc(u8),
    ExprMissingEnd,
    InvalidInstr(u8),
    InvalidValueStack,
    InvalidLocalIdx,
    EmptyValueStack,
    InvalidBinOpTypes(ValType, ValType),
}

pub type Result<T> = core::result::Result<T, Error>;
