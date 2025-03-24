use alloc::format;
use alloc::string::{String, ToString};

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
    DivideBy0,
    UnrepresentableResult,
    FunctionNotFound,
    StackSmash,
    // https://github.com/wasmi-labs/wasmi/blob/37d1449524a322817c55026eb21eb97dd693b9ce/crates/core/src/trap.rs#L265C5-L265C27
    BadConversionToInteger,
    MemoryAccessOutOfBounds,
    TableAccessOutOfBounds,
    ElementAccessOutOfBounds,
    UninitializedElement,
    SignatureMismatch,
    ExpectedAValueOnTheStack,
    ModuleNotFound,
    UnmetImport,
    UndefinedTableIndex,
    // "undefined element" <- as-call_indirect-last
    // "unreachable"
    StoreNotFound,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Proposal {
    Memory64,
    MultipleMemories,
    Threads,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum StoreInstantiationError {
    ActiveDataWriteOutOfBounds,
    I64ValueOutOfReach(String),
    MissingValueOnTheStack,
    TooManyMemories(usize),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LinkerError {
    UnmetImport,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Error {
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
    InvalidMultiByteInstr(u8, u8),
    EndInvalidValueStack,
    InvalidLocalIdx,
    InvalidValidationStackValType(Option<ValType>),
    InvalidValidationStackType(ValidationStackEntry),
    ExpectedAnOperand,
    InvalidLimitsType(u8),
    InvalidMutType(u8),
    MoreThanOneMemory,
    InvalidLimit,
    MemSizeTooBig,
    InvalidGlobalIdx(GlobalIdx),
    GlobalIsConst,
    RuntimeError(RuntimeError),
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
    StoreInstantiationError(StoreInstantiationError),
    OnlyFuncRefIsAllowed,
    TypeUnificationMismatch,
    InvalidSelectTypeVector,
    TooManyLocals(usize),
    UnsupportedProposal(Proposal),
    Overflow,
    LinkerError(LinkerError),
    UnknownFunction,
    UnknownMemory,
    UnknownGlobal,
    DuplicateExportName,
    InvalidImportType,
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
            Error::InvalidSection(section, reason) => f.write_fmt(format_args!(
                "Section '{:?}' invalid! Reason: {}",
                section, reason
            )),
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
            Error::InvalidFuncTypeIdx => {
                f.write_str("An invalid index to the fuctypes table was read")
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
            Error::InvalidValidationStackType(ty) => f.write_fmt(format_args!(
                "An unexpected type was found on the stack: `{ty:?}`"
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
            Error::InvalidLimit => f.write_str("Size minimum must not be greater than maximum"),
            Error::MemSizeTooBig => f.write_str("Memory size must be at most 65536 pages (4GiB)"),
            Error::InvalidGlobalIdx(idx) => f.write_fmt(format_args!(
                "An invalid global index `{idx}` was specified"
            )),
            Error::GlobalIsConst => f.write_str("A const global cannot be written to"),
            Error::RuntimeError(err) => err.fmt(f),
            Error::ExpectedAnOperand => f.write_str("Expected a ValType"), // Error => f.write_str("Expected an operand (ValType) on the stack")
            Error::MemoryIsNotDefined(memidx) => f.write_fmt(format_args!(
                "C.mems[{}] is NOT defined when it should be",
                memidx
            )),
            Error::ErroneousAlignment(mem_align, minimum_wanted_alignment) => {
                f.write_fmt(format_args!(
                    "Alignment ({}) is not less or equal to {}",
                    mem_align, minimum_wanted_alignment
                ))
            }
            Error::NoDataSegments => f.write_str("Data Count is None"),
            Error::DataSegmentNotFound(data_idx) => {
                f.write_fmt(format_args!("Data Segment {} not found", data_idx))
            }
            Error::InvalidLabelIdx(label_idx) => {
                f.write_fmt(format_args!("invalid label index {}", label_idx))
            }
            Error::ValidationCtrlStackEmpty => {
                f.write_str("cannot retrieve last ctrl block, validation ctrl stack is empty")
            }
            Error::ElseWithoutMatchingIf => {
                f.write_str("read 'else' without a previous matching 'if' instruction")
            }
            Error::IfWithoutMatchingElse => {
                f.write_str("read 'end' without matching 'else' instruction to 'if' instruction")
            }
            Error::TableIsNotDefined(table_idx) => f.write_fmt(format_args!(
                "C.tables[{}] is NOT defined when it should be",
                table_idx
            )),
            Error::ElementIsNotDefined(elem_idx) => f.write_fmt(format_args!(
                "C.elems[{}] is NOT defined when it should be",
                elem_idx
            )),
            Error::DifferentRefTypes(rref1, rref2) => f.write_fmt(format_args!(
                "RefType {:?} is NOT equal to RefType {:?}",
                rref1, rref2
            )),
            Error::ExpectedARefType(found_valtype) => f.write_fmt(format_args!(
                "Expected a RefType, found a {:?} instead",
                found_valtype
            )),
            Error::WrongRefTypeForInteropValue(ref_given, ref_wanted) => f.write_fmt(format_args!(
                "Wrong RefType for InteropValue: Given {:?} - Needed {:?}",
                ref_given, ref_wanted
            )),
            Error::FunctionIsNotDefined(func_idx) => f.write_fmt(format_args!(
                "C.functions[{}] is NOT defined when it should be",
                func_idx
            )),
            Error::ReferencingAnUnreferencedFunction(func_idx) => f.write_fmt(format_args!(
                "Referenced a function ({}) that was not referenced in validation",
                func_idx
            )),
            Error::FunctionTypeIsNotDefined(func_ty_idx) => f.write_fmt(format_args!(
                "C.fn_types[{}] is NOT defined when it should be",
                func_ty_idx
            )),
            Error::StoreInstantiationError(err) => err.fmt(f),
            Error::OnlyFuncRefIsAllowed => f.write_str("Only FuncRef is allowed"),
            Error::TypeUnificationMismatch => {
                f.write_str("cannot unify types")
            }
            Error::InvalidSelectTypeVector => {
                f.write_str("SELECT T* (0x1C) instruction must have exactly one type in the subsequent type vector")
            }
            Error::TooManyLocals(x) => {
                f.write_fmt(format_args!("Too many locals (more than 2^32-1): {}", x))
            }
            Error::UnsupportedProposal(proposal) => {
                f.write_fmt(format_args!("Unsupported proposal: {:?}", proposal))
            }
            Error::Overflow => f.write_str("Overflow"),
            Error::LinkerError(err) => err.fmt(f),

            // TODO: maybe move these to LinkerError also add more info to them (the name's export, function idx, etc)
            Error::UnknownFunction => f.write_str("Unknown function"),
            Error::UnknownMemory => f.write_str("Unknown memory"),
            Error::UnknownGlobal => f.write_str("Unknown global"),
            Error::UnknownTable => f.write_str("Unknown table"),
            Error::DuplicateExportName => f.write_str("Duplicate export name"),
            Error::InvalidImportType => f.write_str("Invalid import type")
            // TODO: maybe move these to LinkerError also add more info to them (the name's export, function idx, etc)

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
            RuntimeError::MemoryAccessOutOfBounds => f.write_str("Memory access out of bounds"),
            RuntimeError::TableAccessOutOfBounds => f.write_str("Table access out of bounds"),
            RuntimeError::ElementAccessOutOfBounds => f.write_str("Element access out of bounds"),
            RuntimeError::UninitializedElement => f.write_str("Uninitialized element"),
            RuntimeError::SignatureMismatch => f.write_str("Indirect call signature mismatch"),
            RuntimeError::ExpectedAValueOnTheStack => {
                f.write_str("Expected a value on the stack, but None was found")
            }
            RuntimeError::ModuleNotFound => f.write_str("No such module exists"),
            RuntimeError::UnmetImport => {
                f.write_str("There is at least one import which has no corresponding export")
            }
            RuntimeError::UndefinedTableIndex => {
                f.write_str("Indirect call: table index out of bounds")
            }
            RuntimeError::StoreNotFound => f.write_str("Store not found"),
        }
    }
}

impl Display for StoreInstantiationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        use StoreInstantiationError::*;
        match self {
            ActiveDataWriteOutOfBounds => {
                f.write_str("Active data writing in memory is out of bounds")
            }
            I64ValueOutOfReach(s) => f.write_fmt(format_args!(
                "I64 value {}is out of reach",
                if !s.is_empty() {
                    format!("for {s} ")
                } else {
                    "".to_string()
                }
            )),
            MissingValueOnTheStack => f.write_str(""),
            TooManyMemories(x) => f.write_fmt(format_args!("Too many memories (overflow): {}", x)),
        }
    }
}

impl Display for LinkerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        use LinkerError::*;
        match self {
            UnmetImport => f.write_str("Unmet import"),
        }
    }
}

pub type Result<T> = core::result::Result<T, Error>;

impl From<RuntimeError> for Error {
    fn from(value: RuntimeError) -> Self {
        Self::RuntimeError(value)
    }
}

impl From<StoreInstantiationError> for Error {
    fn from(value: StoreInstantiationError) -> Self {
        Self::StoreInstantiationError(value)
    }
}
