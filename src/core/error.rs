use alloc::string::String;

use crate::core::indices::GlobalIdx;
use crate::validation_stack::ValidationStackEntry;
use crate::RefType;
use core::fmt::{Display, Formatter};
use core::str::Utf8Error;

use crate::core::reader::section_header::SectionTy;
use crate::core::reader::types::ValType;

use super::indices::{DataIdx, ElemIdx, FuncIdx, MemIdx, TableIdx, TypeIdx};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RuntimeError {
    Trap(TrapError),

    ModuleNotFound,
    FunctionNotFound,
    StackExhaustion,
    HostFunctionSignatureMismatch,

    // Are all of these instantiation variants? Add a new `InstantiationError` enum?
    InvalidImportType,
    UnknownImport,
    MoreThanOneMemory,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TrapError {
    DivideBy0,
    UnrepresentableResult,
    // https://github.com/wasmi-labs/wasmi/blob/37d1449524a322817c55026eb21eb97dd693b9ce/crates/core/src/trap.rs#L265C5-L265C27
    BadConversionToInteger,

    /// An access to a memory or data was out of bounds.
    ///
    /// Note: As of now, there is no way to distinguish between both of these. The reference
    /// interpreter and Wast testsuite messages call this error "memory access out of bounds".
    MemoryOrDataAccessOutOfBounds,
    /// An access to a table or an element was out of bounds.
    ///
    /// Note: As of now, there is no way to distinguish between both of these. The reference
    /// interpreter and Wast testsuite messages call this error "table access out of bounds".
    TableOrElementAccessOutOfBounds,
    UninitializedElement,
    SignatureMismatch,
    IndirectCallNullFuncRef,
    TableAccessOutOfBounds,
    ReachedUnreachable,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ValidationError {
    /// The magic number at the very start of the given WASM file is invalid.
    InvalidMagic,
    InvalidVersion,
    MalformedUtf8String(Utf8Error),
    Eof,
    InvalidSection(SectionTy, String),
    InvalidSectionType(u8),
    SectionOutOfOrder(SectionTy),
    InvalidNumType,
    InvalidVecType,
    InvalidFuncType,
    InvalidFuncTypeIdx,
    InvalidRefType,
    InvalidValType,
    InvalidExportDesc(u8),
    InvalidImportDesc(u8),
    ExprMissingEnd,
    InvalidInstr(u8),
    InvalidMultiByteInstr(u8, u32),
    EndInvalidValueStack,
    InvalidLocalIdx,
    InvalidValidationStackValType(Option<ValType>),
    InvalidValidationStackType(ValidationStackEntry),
    ExpectedAnOperand,
    InvalidLimitsType(u8),
    InvalidMutType(u8),
    InvalidLimit,
    MemSizeTooBig,
    InvalidGlobalIdx(GlobalIdx),
    GlobalIsConst,
    MemoryIsNotDefined(MemIdx),
    //           mem.align, wanted alignment
    ErroneousAlignment(u32, u32),
    NoDataSegments,
    DataSegmentNotFound(DataIdx),
    InvalidLabelIdx(usize),
    ValidationCtrlStackEmpty,
    ElseWithoutMatchingIf,
    IfWithoutMatchingElse,
    UnknownTable,
    TableIsNotDefined(TableIdx),
    ElementIsNotDefined(ElemIdx),
    DifferentRefTypes(RefType, RefType),
    ExpectedARefType(ValType),
    WrongRefTypeForInteropValue(RefType, RefType),
    FunctionIsNotDefined(FuncIdx),
    ReferencingAnUnreferencedFunction(FuncIdx),
    FunctionTypeIsNotDefined(TypeIdx),
    OnlyFuncRefIsAllowed,
    TypeUnificationMismatch,
    InvalidSelectTypeVector,
    TooManyLocals(usize),
    Overflow,
    UnknownFunction,
    UnknownMemory,
    UnknownGlobal,
    DuplicateExportName,
    UnsupportedMultipleMemoriesProposal,
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            ValidationError::InvalidMagic => {
                f.write_str("The magic number at the very start of the given WASM file is invalid.")
            }
            ValidationError::InvalidVersion => f.write_str("The version in the WASM file header is invalid"),
            ValidationError::MalformedUtf8String(err) => f.write_fmt(format_args!(
                "A name could not be parsed as it was invalid UTF8: {err}"
            )),
            ValidationError::Eof => f.write_str(
                "A value was expected in the WASM binary but the end of file was reached instead",
            ),
            ValidationError::InvalidSection(section, reason) => f.write_fmt(format_args!(
                "Section '{section:?}' invalid! Reason: {reason}"
            )),
            ValidationError::InvalidSectionType(ty) => f.write_fmt(format_args!(
                "An invalid section type id was found in a section header: {ty}"
            )),
            ValidationError::SectionOutOfOrder(ty) => {
                f.write_fmt(format_args!("The section {ty:?} is out of order"))
            }
            ValidationError::InvalidNumType => {
                f.write_str("An invalid byte was read where a numtype was expected")
            }
            ValidationError::InvalidVecType => {
                f.write_str("An invalid byte was read where a vectype was expected")
            }
            ValidationError::InvalidFuncType => {
                f.write_str("An invalid byte was read where a functype was expected")
            }
            ValidationError::InvalidFuncTypeIdx => {
                f.write_str("An invalid index to the fuctypes table was read")
            }
            ValidationError::InvalidRefType => {
                f.write_str("An invalid byte was read where a reftype was expected")
            }
            ValidationError::InvalidValType => {
                f.write_str("An invalid byte was read where a valtype was expected")
            }
            ValidationError::InvalidExportDesc(byte) => f.write_fmt(format_args!(
                "An invalid byte `{byte:#x?}` was read where an exportdesc was expected"
            )),
            ValidationError::InvalidImportDesc(byte) => f.write_fmt(format_args!(
                "An invalid byte `{byte:#x?}` was read where an importdesc was expected"
            )),
            ValidationError::ExprMissingEnd => f.write_str("An expr is missing an end byte"),
            ValidationError::InvalidInstr(byte) => f.write_fmt(format_args!(
                "An invalid instruction opcode was found: `{byte:#x?}`"
            )),
            ValidationError::InvalidMultiByteInstr(byte1, byte2) => f.write_fmt(format_args!(
                "An invalid multi-byte instruction opcode was found: `{byte1:#x?} {byte2:#x?}`"
            )),
            ValidationError::EndInvalidValueStack => f.write_str(
                "Different value stack types were expected at the end of a block/function.",
            ),
            ValidationError::InvalidLocalIdx => f.write_str("An invalid localidx was used"),
            ValidationError::InvalidValidationStackValType(ty) => f.write_fmt(format_args!(
                "An unexpected type was found on the stack when trying to pop another: `{ty:?}`"
            )),
            ValidationError::InvalidValidationStackType(ty) => f.write_fmt(format_args!(
                "An unexpected type was found on the stack: `{ty:?}`"
            )),
            ValidationError::InvalidLimitsType(ty) => {
                f.write_fmt(format_args!("An invalid limits type was found: {ty:#x?}"))
            }
            ValidationError::InvalidMutType(byte) => f.write_fmt(format_args!(
                "An invalid mut/const byte was found: {byte:#x?}"
            )),
            ValidationError::InvalidLimit => f.write_str("Size minimum must not be greater than maximum"),
            ValidationError::MemSizeTooBig => f.write_str("Memory size must be at most 65536 pages (4GiB)"),
            ValidationError::InvalidGlobalIdx(idx) => f.write_fmt(format_args!(
                "An invalid global index `{idx}` was specified"
            )),
            ValidationError::GlobalIsConst => f.write_str("A const global cannot be written to"),
            ValidationError::ExpectedAnOperand => f.write_str("Expected a ValType"), // Error => f.write_str("Expected an operand (ValType) on the stack")
            ValidationError::MemoryIsNotDefined(memidx) => f.write_fmt(format_args!(
                "C.mems[{memidx}] is NOT defined when it should be"
            )),
            ValidationError::ErroneousAlignment(mem_align, minimum_wanted_alignment) => {
                f.write_fmt(format_args!(
                    "Alignment ({mem_align}) is not less or equal to {minimum_wanted_alignment}"
                ))
            }
            ValidationError::NoDataSegments => f.write_str("Data Count is None"),
            ValidationError::DataSegmentNotFound(data_idx) => {
                f.write_fmt(format_args!("Data Segment {data_idx} not found"))
            }
            ValidationError::InvalidLabelIdx(label_idx) => {
                f.write_fmt(format_args!("invalid label index {label_idx}"))
            }
            ValidationError::ValidationCtrlStackEmpty => {
                f.write_str("cannot retrieve last ctrl block, validation ctrl stack is empty")
            }
            ValidationError::ElseWithoutMatchingIf => {
                f.write_str("read 'else' without a previous matching 'if' instruction")
            }
            ValidationError::IfWithoutMatchingElse => {
                f.write_str("read 'end' without matching 'else' instruction to 'if' instruction")
            }
            ValidationError::TableIsNotDefined(table_idx) => f.write_fmt(format_args!(
                "C.tables[{table_idx}] is NOT defined when it should be"
            )),
            ValidationError::ElementIsNotDefined(elem_idx) => f.write_fmt(format_args!(
                "C.elems[{elem_idx}] is NOT defined when it should be"
            )),
            ValidationError::DifferentRefTypes(rref1, rref2) => f.write_fmt(format_args!(
                "RefType {rref1:?} is NOT equal to RefType {rref2:?}"
            )),
            ValidationError::ExpectedARefType(found_valtype) => f.write_fmt(format_args!(
                "Expected a RefType, found a {found_valtype:?} instead"
            )),
            ValidationError::WrongRefTypeForInteropValue(ref_given, ref_wanted) => f.write_fmt(format_args!(
                "Wrong RefType for InteropValue: Given {ref_given:?} - Needed {ref_wanted:?}"
            )),
            ValidationError::FunctionIsNotDefined(func_idx) => f.write_fmt(format_args!(
                "C.functions[{func_idx}] is NOT defined when it should be"
            )),
            ValidationError::ReferencingAnUnreferencedFunction(func_idx) => f.write_fmt(format_args!(
                "Referenced a function ({func_idx}) that was not referenced in validation"
            )),
            ValidationError::FunctionTypeIsNotDefined(func_ty_idx) => f.write_fmt(format_args!(
                "C.fn_types[{func_ty_idx}] is NOT defined when it should be"
            )),
            ValidationError::OnlyFuncRefIsAllowed => f.write_str("Only FuncRef is allowed"),
            ValidationError::TypeUnificationMismatch => {
                f.write_str("cannot unify types")
            }
            ValidationError::InvalidSelectTypeVector => {
                f.write_str("SELECT T* (0x1C) instruction must have exactly one type in the subsequent type vector")
            }
            ValidationError::TooManyLocals(x) => {
                f.write_fmt(format_args!("Too many locals (more than 2^32-1): {x}"))
            }
            ValidationError::Overflow => f.write_str("Overflow"),
            // TODO: maybe move these to LinkerError also add more info to them (the name's export, function idx, etc)
            ValidationError::UnknownFunction => f.write_str("Unknown function"),
            ValidationError::UnknownMemory => f.write_str("Unknown memory"),
            ValidationError::UnknownGlobal => f.write_str("Unknown global"),
            ValidationError::UnknownTable => f.write_str("Unknown table"),
            ValidationError::DuplicateExportName => f.write_str("Duplicate export name"),
            ValidationError::UnsupportedMultipleMemoriesProposal => f.write_str("Proposal for multiple memories is not yet supported"),
        }
    }
}

impl Display for TrapError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            TrapError::DivideBy0 => f.write_str("Divide by zero is not permitted"),
            TrapError::UnrepresentableResult => f.write_str("Result is unrepresentable"),
            TrapError::BadConversionToInteger => f.write_str("Bad conversion to integer"),
            TrapError::MemoryOrDataAccessOutOfBounds => {
                f.write_str("Memory or data access out of bounds")
            }
            TrapError::TableOrElementAccessOutOfBounds => {
                f.write_str("Table or element access out of bounds")
            }
            TrapError::UninitializedElement => f.write_str("Uninitialized element"),
            TrapError::SignatureMismatch => f.write_str("Indirect call signature mismatch"),
            TrapError::IndirectCallNullFuncRef => {
                f.write_str("Indirect call targeted null reference")
            }
            TrapError::TableAccessOutOfBounds => {
                f.write_str("Indirect call: table index out of bounds")
            }
            TrapError::ReachedUnreachable => {
                f.write_str("an unreachable statement was reached, triggered a trap")
            }
        }
    }
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            RuntimeError::Trap(trap_error) => write!(f, "{trap_error}"),
            RuntimeError::FunctionNotFound => f.write_str("Function not found"),
            RuntimeError::ModuleNotFound => f.write_str("No such module exists"),
            RuntimeError::StackExhaustion => {
                f.write_str("either the call stack or the value stack overflowed")
            }
            RuntimeError::HostFunctionSignatureMismatch => {
                f.write_str("host function call did not respect its type signature")
            }
            RuntimeError::InvalidImportType => f.write_str("Invalid import type"),
            // TODO: maybe move these to LinkerError also add more info to them (the name's export, function idx, etc)
            RuntimeError::UnknownImport => f.write_str("Unknown Import"),
            RuntimeError::MoreThanOneMemory => {
                f.write_str("As of not only one memory is allowed per module.")
            }
        }
    }
}

impl From<TrapError> for RuntimeError {
    fn from(value: TrapError) -> Self {
        Self::Trap(value)
    }
}

/// A definition for a [`Result`] using the optional [`Error`] type.
pub type Result<T> = core::result::Result<T, Error>;

/// An opt-in error type useful for merging all error types of this crate into a single type.
///
/// Note: This crate does not use this type in any public interfaces, making it optional for downstream users.
pub enum Error {
    Validation(ValidationError),
    RuntimeError(RuntimeError),
}

impl From<ValidationError> for Error {
    fn from(value: ValidationError) -> Self {
        Self::Validation(value)
    }
}

impl From<RuntimeError> for Error {
    fn from(value: RuntimeError) -> Self {
        Self::RuntimeError(value)
    }
}
