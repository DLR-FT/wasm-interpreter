use crate::core::indices::GlobalIdx;
use crate::validation_stack::LabelKind;
use core::fmt::{Display, Formatter};
use core::str::Utf8Error;

use crate::core::reader::section_header::SectionTy;
use crate::core::reader::types::ValType;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RuntimeError {
    DivideBy0,
    UnrepresentableResult,
    FunctionNotFound,
    StackSmash,
    // https://github.com/wasmi-labs/wasmi/blob/37d1449524a322817c55026eb21eb97dd693b9ce/crates/core/src/trap.rs#L265C5-L265C27
    BadConversionToInteger,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Error {
    /// The magic number at the very start of the given WASM file is invalid.
    InvalidMagic,
    InvalidVersion,
    MalformedUtf8String(Utf8Error),
    Eof,
    InvalidSectionType(u8),
    SectionOutOfOrder(SectionTy),
    InvalidNumType,
    InvalidVecType,
    InvalidFuncType,
    InvalidRefType,
    InvalidValType,
    InvalidExportDesc(u8),
    InvalidImportDesc(u8),
    ExprMissingEnd,
    InvalidInstr(u8),
    InvalidMultiByteInstr(u8, u8),
    EndInvalidValueStack,
    InvalidLocalIdx,
    InvalidValidationStackValType(Option<ValType>),
    InvalidLimitsType(u8),
    InvalidMutType(u8),
    MoreThanOneMemory,
    InvalidGlobalIdx(GlobalIdx),
    GlobalIsConst,
    RuntimeError(RuntimeError),
    FoundLabel(LabelKind),
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
            Error::Eof => f.write_str(
                "A value was expected in the WASM binary but the end of file was reached instead",
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
            Error::InvalidImportDesc(byte) => f.write_fmt(format_args!(
                "An invalid byte `{byte:#x?}` was read where an importdesc was expected"
            )),
            Error::ExprMissingEnd => f.write_str("An expr is missing an end byte"),
            Error::InvalidInstr(byte) => f.write_fmt(format_args!(
                "An invalid instruction opcode was found: `{byte:#x?}`"
            )),
            Error::InvalidMultiByteInstr(byte1, byte2) => f.write_fmt(format_args!(
                "An invalid multi-byte instruction opcode was found: `{byte1:#x?} {byte2:#x?}`"
            )),
            Error::EndInvalidValueStack => f.write_str(
                "Different value stack types were expected at the end of a block/function.",
            ),
            Error::InvalidLocalIdx => f.write_str("An invalid localidx was used"),
            Error::InvalidValidationStackValType(ty) => f.write_fmt(format_args!(
                "An unexpected type was found on the stack when trying to pop another: `{ty:?}`"
            )),
            Error::InvalidLimitsType(ty) => {
                f.write_fmt(format_args!("An invalid limits type was found: {ty:#x?}"))
            }
            Error::InvalidMutType(byte) => f.write_fmt(format_args!(
                "An invalid mut/const byte was found: {byte:#x?}"
            )),
            Error::MoreThanOneMemory => {
                f.write_str("As of not only one memory is allowed per module.")
            }
            Error::InvalidGlobalIdx(idx) => f.write_fmt(format_args!(
                "An invalid global index `{idx}` was specified"
            )),
            Error::GlobalIsConst => f.write_str("A const global cannot be written to"),
            Error::RuntimeError(err) => err.fmt(f),
            Error::FoundLabel(lk) => f.write_fmt(format_args!(
                "Expecting a ValType, a Label was found: {lk:?}"
            )),
        }
    }
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            RuntimeError::DivideBy0 => f.write_str("Divide by zero is not permitted"),
            RuntimeError::UnrepresentableResult => f.write_str("Result is unrepresentable"),
            RuntimeError::FunctionNotFound => f.write_str("Function not found"),
            RuntimeError::StackSmash => f.write_str("Stack smashed"),
            RuntimeError::BadConversionToInteger => f.write_str("Bad conversion to integer"),
        }
    }
}

pub type Result<T> = core::result::Result<T, Error>;

impl From<RuntimeError> for Error {
    fn from(value: RuntimeError) -> Self {
        Self::RuntimeError(value)
    }
}
