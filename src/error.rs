use core::fmt::Display;
use core::str::Utf8Error;

use crate::section::SectionTy;

#[derive(Debug)]
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
    InvalidInstr,
}

pub type Result<T> = core::result::Result<T, Error>;
