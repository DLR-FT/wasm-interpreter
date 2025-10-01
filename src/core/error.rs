use alloc::string::String;

use crate::core::indices::GlobalIdx;
use crate::validation_stack::ValidationStackEntry;
use crate::RefType;
use core::fmt::{Display, Formatter};
use core::str::Utf8Error;

use crate::core::reader::section_header::SectionTy;
use crate::core::reader::types::ValType;

use super::indices::{DataIdx, ElemIdx, FuncIdx, LabelIdx, LocalIdx, MemIdx, TableIdx, TypeIdx};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ValidationError {
    /// The magic number at the start of the Wasm bytecode is invalid.
    InvalidMagic,
    /// The Wasm binary version at the start of the Wasm bytecode is invalid.
    InvalidVersion,
    /// A UTF-8 string was invalid.
    MalformedUtf8String(Utf8Error),
    /// The end of the binary file was reached unexpectedly.
    Eof,

    /// An index for a type is invalid.
    InvalidTypeIdx(TypeIdx),
    /// An index for a function is invalid.
    InvalidFuncIdx(FuncIdx),
    /// An index for a table is invalid.
    InvalidTableIdx(TableIdx),
    /// An index for a memory is invalid.
    InvalidMemIndex(MemIdx),
    /// An index for a global is invalid.
    InvalidGlobalIdx(GlobalIdx),
    /// An index for an element segment is invalid.
    InvalidElemIdx(ElemIdx),
    /// An index for a data segment is invalid.
    InvalidDataIdx(DataIdx),
    /// An index for a local is invalid.
    InvalidLocalIdx(LocalIdx),
    /// An index for a label is invalid.
    InvalidLabelIdx(LabelIdx),
    /// An index for a lane of some vector type is invalid.
    InvalidLaneIdx(u8),

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
    InvalidValidationStackValType(Option<ValType>),
    InvalidValidationStackType(ValidationStackEntry),
    ExpectedAnOperand,
    InvalidLimitsType(u8),
    InvalidMutType(u8),
    InvalidLimit,
    MemSizeTooBig,
    GlobalIsConst,
    //           mem.align, wanted alignment
    ErroneousAlignment(u32, u32),
    NoDataSegments,
    ValidationCtrlStackEmpty,
    ElseWithoutMatchingIf,
    IfWithoutMatchingElse,
    UnknownTable,
    DifferentRefTypes(RefType, RefType),
    ExpectedARefType(ValType),
    WrongRefTypeForInteropValue(RefType, RefType),
    ReferencingAnUnreferencedFunction(FuncIdx),
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
    ExprHasTrailingInstructions,
    FunctionAndCodeSectionsHaveDifferentLengths,
    DataCountAndDataSectionsLengthAreDifferent,
    InvalidImportType,
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

            ValidationError::InvalidTypeIdx(idx) => write!(f, "invalid type index {idx}"),
            ValidationError::InvalidFuncIdx(idx) => write!(f, "invalid func index {idx}"),
            ValidationError::InvalidTableIdx(idx) => write!(f, "invalid table index {idx}"),
            ValidationError::InvalidMemIndex(idx) => write!(f, "invalid memory index {idx}"),
            ValidationError::InvalidGlobalIdx(idx) => write!(f, "invalid global index {idx}"),
            ValidationError::InvalidElemIdx(idx) => write!(f, "invalid element segment index {idx}"),
            ValidationError::InvalidDataIdx(idx) => write!(f, "invalid data segment index {idx}"),
            ValidationError::InvalidLocalIdx(idx) => write!(f, "invalid local index {idx}"),
            ValidationError::InvalidLabelIdx(idx) => write!(f, "invalid label index {idx}"),
            ValidationError::InvalidLaneIdx(idx) => write!(f, "invalid lane index {idx}"),

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
            ValidationError::GlobalIsConst => f.write_str("A const global cannot be written to"),
            ValidationError::ExpectedAnOperand => f.write_str("Expected a ValType"), // Error => f.write_str("Expected an operand (ValType) on the stack")
            ValidationError::ErroneousAlignment(mem_align, minimum_wanted_alignment) => {
                f.write_fmt(format_args!(
                    "Alignment ({mem_align}) is not less or equal to {minimum_wanted_alignment}"
                ))
            }
            ValidationError::NoDataSegments => f.write_str("Data Count is None"),
            ValidationError::ValidationCtrlStackEmpty => {
                f.write_str("cannot retrieve last ctrl block, validation ctrl stack is empty")
            }
            ValidationError::ElseWithoutMatchingIf => {
                f.write_str("read 'else' without a previous matching 'if' instruction")
            }
            ValidationError::IfWithoutMatchingElse => {
                f.write_str("read 'end' without matching 'else' instruction to 'if' instruction")
            }
            ValidationError::DifferentRefTypes(rref1, rref2) => f.write_fmt(format_args!(
                "RefType {rref1:?} is NOT equal to RefType {rref2:?}"
            )),
            ValidationError::ExpectedARefType(found_valtype) => f.write_fmt(format_args!(
                "Expected a RefType, found a {found_valtype:?} instead"
            )),
            ValidationError::WrongRefTypeForInteropValue(ref_given, ref_wanted) => f.write_fmt(format_args!(
                "Wrong RefType for InteropValue: Given {ref_given:?} - Needed {ref_wanted:?}"
            )),
            ValidationError::ReferencingAnUnreferencedFunction(func_idx) => f.write_fmt(format_args!(
                "Referenced a function ({func_idx}) that was not referenced in validation"
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
            ValidationError::ExprHasTrailingInstructions => f.write_str("A code expression has invalid trailing instructions"),
            ValidationError::FunctionAndCodeSectionsHaveDifferentLengths => f.write_str("The function and code sections have different lengths"),
            ValidationError::DataCountAndDataSectionsLengthAreDifferent => f.write_str("The data count section specifies a different length than there are actual segments in the data section."),
            ValidationError::InvalidImportType => f.write_str("Invalid import type"),
        }
    }
}

#[cfg(test)]
mod test {
    use alloc::string::ToString;

    use crate::ValidationError;

    #[test]
    fn fmt_invalid_magic() {
        assert!(ValidationError::InvalidMagic
            .to_string()
            .contains("magic number"));
    }

    #[test]
    fn fmt_invalid_version() {
        assert!(ValidationError::InvalidVersion
            .to_string()
            .contains("version"));
    }
}
