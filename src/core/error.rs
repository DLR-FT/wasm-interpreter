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
    /// The binary format version at the start of the Wasm bytecode is invalid.
    InvalidBinaryFormatVersion,
    /// The end of the binary file was reached unexpectedly.
    Eof,

    /// A UTF-8 string is malformed.
    MalformedUtf8(Utf8Error),
    /// The type of a section is malformed.
    MalformedSectionTypeDiscriminator(u8),
    /// The discriminator of a number type is malformed.
    MalformedNumTypeDiscriminator(u8),
    /// The discriminator of a vector type is malformed.
    MalformedVecTypeDiscriminator(u8),
    /// The discriminator of a function type is malformed.
    MalformedFuncTypeDiscriminator(u8),
    /// The discriminator of a reference type is malformed.
    MalformedRefTypeDiscriminator(u8),
    /// A valtype is malformed because it is neither a number, reference nor vector type.
    MalformedValType,
    /// The discriminator of an export description is malformed.
    MalformedExportDescDiscriminator(u8),
    /// The discriminator of an import description is malformed.
    MalformedImportDescDiscriminator(u8),
    /// The discriminator of a limits type is malformed.
    MalformedLimitsDiscriminator(u8),
    /// The min field of a limits type is larger than the max field.
    MalformedLimitsMinLargerThanMax {
        min: u32,
        max: u32,
    },
    /// The discriminator of a mut type is malformed.
    MalformedMutDiscriminator(u8),
    /// Block types use a special 33-bit signed integer for encoding type indices.
    MalformedBlockTypeTypeIdx(i64),
    /// A variable-length integer was read but it overflowed.
    MalformedVariableLengthInteger,
    /// The discriminator of an element kind is malformed.
    MalformedElemKindDiscriminator(u8),

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

    /// A section with given type is out of order. All section types have a fixed order in which they must occur.
    SectionOutOfOrder(SectionTy),
    /// A custom section contains more bytes than its section header specifies.
    InvalidCustomSectionLength,
    ExprMissingEnd,
    InvalidInstr(u8),
    InvalidMultiByteInstr(u8, u32),
    EndInvalidValueStack,
    InvalidValidationStackValType(Option<ValType>),
    InvalidValidationStackType(ValidationStackEntry),
    ExpectedAnOperand,
    /// The memory size specified by a mem type exceeds the maximum size.
    MemoryTooLarge,
    /// An attempt has been made to mutate a const global
    MutationOfConstGlobal,
    /// An alignment of some memory instruction is invalid
    ErroneousAlignment {
        alignment: u32,
        minimum_required_alignment: u32,
    },
    /// The validation control stack is empty, even though an entry was expected.
    // TODO Reconsider if we want to expose this error. It should probably never happen and thus also never bubble up to the user.
    ValidationCtrlStackEmpty,
    /// An `else` instruction was found while not inside an `if` block.
    ElseWithoutMatchingIf,
    /// An `end` for a matching `if` instruction was found, but there was no `else` instruction in between.
    IfWithoutMatchingElse,
    /// A `table.init` instruction specified a table and an element segment that store different reference types.
    MismatchedRefTypesDuringTableInit {
        table_ty: RefType,
        elem_ty: RefType,
    },
    /// A `table.copy` instruction referenced two tables that store different reference types.
    MismatchedRefTypesDuringTableCopy {
        source_table_ty: RefType,
        destination_table_ty: RefType,
    },
    /// An expected reference type did not match the actual reference type on the validation stack.
    MismatchedRefTypesOnValidationStack {
        expected: RefType,
        actual: RefType,
    },
    /// An indirect call to a table with does not store function references was made.
    IndirectCallToNonFuncRefTable(RefType),
    /// A reference type was expected to be on the stack, but a value type was found.
    ExpectedReferenceTypeOnStack(ValType),
    /// When a is referenced in the code section it must be contained in `C.refs`, which was not the case
    ReferencingAnUnreferencedFunction(FuncIdx),
    /// The select instructions may work with multiple values in the future. However, as of now its vector may only have one element.
    InvalidSelectTypeVectorLength(usize),
    /// A function specifies too many locals, i.e. more than 2^32 - 1
    TooManyLocals(u64),
    /// Multiple exports share the same name
    DuplicateExportName,
    /// Multiple memories are not yet allowed without the proposal.
    UnsupportedMultipleMemoriesProposal,
    /// An expr in the code section has trailing instructions following its `end` instruction.
    CodeExprHasTrailingInstructions,
    /// The lengths of the function and code sections must match.
    FunctionAndCodeSectionsHaveDifferentLengths,
    /// The data count specified in the data count section and the length of the data section must match.
    DataCountAndDataSectionsLengthAreDifferent,
    InvalidImportType,
    /// The function signature of the start function is invalid. It must not specify any parameters or return values.
    InvalidStartFunctionSignature,
    /// An active element segment's type and its table's type are different.
    ActiveElementSegmentTypeMismatch,
    /// 33-bit signed integers are sometimes used to encode unsigned 32-bit
    /// integers to prevent collisions between bit patterns of different types.
    /// Therefore, 33-bit signed integers may never be negative.
    I33IsNegative,
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            ValidationError::InvalidMagic => write!(f, "The magic number is invalid"),
            ValidationError::InvalidBinaryFormatVersion => write!(f, "The Wasm binary format version is invalid"),
            ValidationError::Eof => write!(f, "The end of the Wasm bytecode was reached unexpectedly"),

            ValidationError::MalformedUtf8(utf8_error) => write!(f, "Failed to parse a UTF-8 string: {utf8_error}"),
            ValidationError::MalformedSectionTypeDiscriminator(byte) => write!(f, "Failed to parse {byte:#x} as a section type discriminator"),
            ValidationError::MalformedNumTypeDiscriminator(byte) => write!(f, "Failed to parse {byte:#x} as a number type discriminator"),
            ValidationError::MalformedVecTypeDiscriminator(byte) => write!(f, "Failed to parse {byte:#x} as a vector type discriminator"),
            ValidationError::MalformedFuncTypeDiscriminator(byte) => write!(f, "Failed to parse {byte:#x} as a function type discriminator"),
            ValidationError::MalformedRefTypeDiscriminator(byte) => write!(f, "Failed to parse {byte:#x} as a reference type discriminator"),
            ValidationError::MalformedValType => write!(f, "Failed to read a value type because it is neither a number, reference or vector type"),
            ValidationError::MalformedExportDescDiscriminator(byte) => write!(f, "Failed to parse {byte:#x} as an export description discriminator"),
            ValidationError::MalformedImportDescDiscriminator(byte) => write!(f, "Failed to parse {byte:#x} as an import description discriminator"),
            ValidationError::MalformedLimitsDiscriminator(byte) => write!(f, "Failed to parse {byte:#x} as a limits type discriminator"),
            ValidationError::MalformedLimitsMinLargerThanMax { min, max } => write!(f, "Limits are malformed because min={min} is larger than max={max}"),
            ValidationError::MalformedMutDiscriminator(byte) => write!(f, "Failed to parse {byte:#x} as a mute type discriminator"),
            ValidationError::MalformedBlockTypeTypeIdx(idx) => write!(f, "The type index {idx} which is encoded as a singed 33-bit integer inside a block type is malformed"),
            ValidationError::MalformedVariableLengthInteger => write!(f, "Reading a variable-length integer overflowed"),
            ValidationError::MalformedElemKindDiscriminator(byte) => write!(f, "Failed to parse {byte:#x} as an element kind discriminator"),

            ValidationError::InvalidTypeIdx(idx) => write!(f, "The type index {idx} is invalid"),
            ValidationError::InvalidFuncIdx(idx) => write!(f, "The function index {idx} is invalid"),
            ValidationError::InvalidTableIdx(idx) => write!(f, "The table index {idx} is invalid"),
            ValidationError::InvalidMemIndex(idx) => write!(f, "The memory index {idx} is invalid"),
            ValidationError::InvalidGlobalIdx(idx) => write!(f, "The global index {idx} is invalid"),
            ValidationError::InvalidElemIdx(idx) => write!(f, "The element segment index {idx} is invalid"),
            ValidationError::InvalidDataIdx(idx) => write!(f, "The data segment index {idx} is invalid"),
            ValidationError::InvalidLocalIdx(idx) => write!(f, "The local index {idx} is invalid"),
            ValidationError::InvalidLabelIdx(idx) => write!(f, "The label index {idx} is invalid"),
            ValidationError::InvalidLaneIdx(idx) => write!(f, "The lane index {idx} is invalid"),

            ValidationError::SectionOutOfOrder(ty) => write!(f, "A section of type `{ty:?}` is defined out of order"),
            ValidationError::InvalidCustomSectionLength => write!(f, "A custom section contains more bytes than its section header specifies"),
            ValidationError::ExprMissingEnd => write!(f, "An expr type is missing an end byte"),
            ValidationError::InvalidInstr(byte) => write!(f, "The instruction opcode {byte:#x} is invalid"),
            ValidationError::InvalidMultiByteInstr(first_byte, second_instr) => write!(f, "The multi-byte instruction opcode {first_byte:#x} {second_instr} is invalid"),
            ValidationError::ActiveElementSegmentTypeMismatch => write!(f, "an element segment's type and its table's type are different"),
            ValidationError::EndInvalidValueStack => write!(f, "Different value stack types were expected at the end of a block/function"),
            ValidationError::InvalidValidationStackValType(ty) => write!(f, "An unexpected type `{ty:?}` was found on the stack when trying to pop another"),
            ValidationError::InvalidValidationStackType(ty) => write!(f, "An unexpected type `{ty:?}` was found on the stack"),
            ValidationError::ExpectedAnOperand => write!(f, "Expected a value type operand on the stack"),
            ValidationError::MemoryTooLarge => write!(f, "The size specified by a memory type exceeds the maximum size"),
            ValidationError::MutationOfConstGlobal => write!(f, "An attempt has been made to mutate a const global"),
            ValidationError::ErroneousAlignment {alignment , minimum_required_alignment} => write!(f, "The alignment 2^{alignment} is not less or equal to the required alignment 2^{minimum_required_alignment}"),
            ValidationError::ValidationCtrlStackEmpty => write!(f, "Failed to retrieve last ctrl block because validation ctrl stack is empty"),
            ValidationError::ElseWithoutMatchingIf => write!(f, "Found `else` without a previous matching `if` instruction"),
            ValidationError::IfWithoutMatchingElse => write!(f, "Found `end` without a previous matching `else` to an `if` instruction"),
            ValidationError::MismatchedRefTypesDuringTableInit { table_ty, elem_ty } => write!(f, "Mismatch of table type `{table_ty:?}` and element segment type `{elem_ty:?}` for `table.init` instruction"),
            ValidationError::MismatchedRefTypesDuringTableCopy { source_table_ty, destination_table_ty } => write!(f, "Mismatch of source table type `{source_table_ty:?}` and destination table type `{destination_table_ty:?}` for `table.copy` instruction"),
            ValidationError::MismatchedRefTypesOnValidationStack { expected, actual } => write!(f, "Mismatch of reference types on the value stack: Expected `{expected:?}` but got `{actual:?}`"),
            ValidationError::IndirectCallToNonFuncRefTable(table_ty) => write!(f, "An indirect call to a table which does not store function references but instead `{table_ty:?}` was made"),
            ValidationError::ExpectedReferenceTypeOnStack(found_valtype) => write!(f, "Expected a reference type but instead found a `{found_valtype:?}` on the stack"),
            ValidationError::ReferencingAnUnreferencedFunction(func_idx) => write!(f, "Referenced a function with index {func_idx} that was not referenced in prior validation"),
            ValidationError::InvalidSelectTypeVectorLength(len) => write!(f, "The type vector of a `select` instruction must be of length 1 as of now but it is of length {len} instead"),
            ValidationError::TooManyLocals(n) => write!(f,"There are {n} locals and this exceeds the maximum allowed number of 2^32-1"),
            ValidationError::DuplicateExportName => write!(f,"Multiple exports share the same name"),
            ValidationError::UnsupportedMultipleMemoriesProposal => write!(f,"A memory index other than 1 was used, but the proposal for multiple memories is not yet supported"),
            ValidationError::CodeExprHasTrailingInstructions => write!(f,"A code expression has invalid trailing instructions following its `end` instruction"),
            ValidationError::FunctionAndCodeSectionsHaveDifferentLengths => write!(f,"The function and code sections have different lengths"),
            ValidationError::DataCountAndDataSectionsLengthAreDifferent => write!(f,"The data count section specifies a different length than there are data segments in the data section"),
            ValidationError::InvalidImportType => f.write_str("Invalid import type"),
            ValidationError::InvalidStartFunctionSignature => write!(f,"The start function has parameters or return types which it is not allowed to have"),
            ValidationError::I33IsNegative => f.write_str("An i33 type is negative which is not allowed")
        }
    }
}

impl ValidationError {
    /// Convert this error to a message that is compatible with the error messages used by the official Wasm testsuite.
    pub fn to_message(&self) -> &'static str {
        todo!("convert validation error to testsuite message");
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
        assert!(ValidationError::InvalidBinaryFormatVersion
            .to_string()
            .contains("version"));
    }
}
