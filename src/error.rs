use alloc::fmt::format;
use core::fmt::{Display, Formatter, Write};
use core::str::Utf8Error;

use crate::section::SectionTy;
use crate::wasm::types::ValType;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Error {
    /// The magic number at the very start of the given WASM file is invalid.
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
    EndInvalidValueStack,
    InvalidLocalIdx,
    InvalidValueStackType(Option<ValType>),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::InvalidMagic => {
                f.write_str("The magic number at the very start of the given WASM file is invalid.")
            }
            Error::InvalidVersion => f.write_str("The version in the WASM file header is invalid"),
            Error::MalformedUtf8String(err) => f.write_fmt(format_args!(
                "A name could not be parsed as it was invalid UTF8: {err}"
            )),
            Error::MissingValue => f.write_str(
                "A value was expected in the WASM binary but it could not be found or read",
            ),
            Error::InvalidSectionType(ty) => f.write_fmt(format_args!(
                "An invalid section type id was found in a section header: {ty}"
            )),
            Error::SectionOutOfOrder(ty) => {
                f.write_fmt(format_args!("The section {ty:?} is out of order"))
            }
            Error::InvalidNumType => {
                f.write_str("An invalid byte was read where a numtype was expected")
            }
            Error::InvalidVecType => {
                f.write_str("An invalid byte was read where a vectype was expected")
            }
            Error::InvalidFuncType => {
                f.write_str("An invalid byte was read where a functype was expected")
            }
            Error::InvalidRefType => {
                f.write_str("An invalid byte was read where a reftype was expected")
            }
            Error::InvalidValType => {
                f.write_str("An invalid byte was read where a valtype was expected")
            }
            Error::InvalidExportDesc(byte) => f.write_fmt(format_args!(
                "An invalid byte `{byte:#x?}` was read where an exportdesc was expected"
            )),
            Error::ExprMissingEnd => f.write_str("An expr is missing an end byte"),
            Error::InvalidInstr(byte) => f.write_fmt(format_args!(
                "An invalid instruction opcode was found: `{byte:#x?}`"
            )),
            Error::EndInvalidValueStack => f.write_str(
                "Different value stack types were expected at the end of a block/function.",
            ),
            Error::InvalidLocalIdx => f.write_str("An invalid localidx was used"),
            Error::InvalidValueStackType(ty) => f.write_fmt(format_args!(
                "An unexpected type was found on the stack when trying to pop another: `{ty:?}`"
            )),
        }
    }
}

pub type Result<T> = core::result::Result<T, Error>;
