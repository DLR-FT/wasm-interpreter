use core::fmt::{Display, Formatter};

use crate::{
    core::structure::{modules::indices::FuncIdx, types::ValType},
    validation::validation_stack::ValidationStackEntry,
    DecodingError, RefType,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ValidationError {
    Decoding(DecodingError),
    /// An index for a type is invalid.
    InvalidTypeIdx(u32),
    /// An index for a function is invalid.
    InvalidFuncIdx(u32),
    /// An index for a table is invalid.
    InvalidTableIdx(u32),
    /// An index for a memory is invalid.
    InvalidMemIdx(u32),
    /// An index for a global is invalid.
    InvalidGlobalIdx(u32),
    /// An index for an element segment is invalid.
    InvalidElemIdx(u32),
    /// An index for a data segment is invalid.
    InvalidDataIdx(u32),
    /// An index for a local is invalid.
    InvalidLocalIdx(u32),
    /// An index for a label is invalid.
    InvalidLabelIdx(u32),
    /// An index for a lane of some vector type is invalid.
    InvalidLaneIdx(u8),

    ExprMissingEnd,
    InvalidInstr(u8),
    InvalidMultiByteInstr(u8, u32),
    EndInvalidValueStack,
    InvalidValidationStackValType(Option<ValType>),
    InvalidValidationStackType(ValidationStackEntry),
    ExpectedAnOperand,
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
    /// The data count section is required, if there are instructions that use
    /// data indices.
    MissingDataCountSection,
    /// The mode of a data segment was invalid. Only values in the range 0..=2
    /// are allowed.
    InvalidDataSegmentMode(u32),
    /// The mode of an element was invalid. Only values in the range 0..=7 are
    /// allowed.
    InvalidElementMode(u32),
    /// The module contains too many functions, i.e. imported or locally-defined
    /// functions. The maximum number of functions is [`u32::MAX`].
    TooManyFunctions,
    /// The module contains too many tables, i.e. imported or locally-defined
    /// tables. The maximum number of tables is [`u32::MAX`].
    TooManyTables,
    /// The module contains too many memories, i.e. imported or locally-defined
    /// memories. The maximum number of memories is [`u32::MAX`].
    TooManyMemories,
    /// The module contains too many globals, i.e. imported or locally-defined
    /// globals. The maximum number of memories is [`u32::MAX`].
    TooManyGlobals,
    /// The min field of a limits type is larger than the max field.
    LimitsMinLargerThanMax {
        min: u32,
        max: u32,
    },
    /// The min or max field of a limits type is not within the expected range.
    LimitsNotWithinRange(u32),
    /// A mutable global was referenced in some global.get instruction in a constant expression.
    MutGlobalInConstGlobalGet,
}

impl core::error::Error for ValidationError {}

impl Display for ValidationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            ValidationError::Decoding(err) => write!(f, "Decoding failed: {err}"),

            ValidationError::InvalidTypeIdx(idx) => write!(f, "The type index {idx} is invalid"),
            ValidationError::InvalidFuncIdx(idx) => write!(f, "The function index {idx} is invalid"),
            ValidationError::InvalidTableIdx(idx) => write!(f, "The table index {idx} is invalid"),
            ValidationError::InvalidMemIdx(idx) => write!(f, "The memory index {idx} is invalid"),
            ValidationError::InvalidGlobalIdx(idx) => write!(f, "The global index {idx} is invalid"),
            ValidationError::InvalidElemIdx(idx) => write!(f, "The element segment index {idx} is invalid"),
            ValidationError::InvalidDataIdx(idx) => write!(f, "The data segment index {idx} is invalid"),
            ValidationError::InvalidLocalIdx(idx) => write!(f, "The local index {idx} is invalid"),
            ValidationError::InvalidLabelIdx(idx) => write!(f, "The label index {idx} is invalid"),
            ValidationError::InvalidLaneIdx(idx) => write!(f, "The lane index {idx} is invalid"),

            ValidationError::ExprMissingEnd => write!(f, "An expr type is missing an end byte"),
            ValidationError::InvalidInstr(byte) => write!(f, "The instruction {byte:#x} is invalid"),
            ValidationError::InvalidMultiByteInstr(first_byte, second_instr) => write!(f, "The multi-byte instruction {first_byte:#x} {second_instr} is invalid"),
            ValidationError::ActiveElementSegmentTypeMismatch => write!(f, "an element segment's type and its table's type are different"),
            ValidationError::EndInvalidValueStack => write!(f, "Different value stack types were expected at the end of a block/function"),
            ValidationError::InvalidValidationStackValType(ty) => write!(f, "An unexpected type `{ty:?}` was found on the stack when trying to pop another"),
            ValidationError::InvalidValidationStackType(ty) => write!(f, "An unexpected type `{ty:?}` was found on the stack"),
            ValidationError::ExpectedAnOperand => write!(f, "Expected a value type operand on the stack"),
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
            ValidationError::DuplicateExportName => write!(f,"Multiple exports share the same name"),
            ValidationError::UnsupportedMultipleMemoriesProposal => write!(f,"A memory index other than 1 was used, but the proposal for multiple memories is not yet supported"),
            ValidationError::CodeExprHasTrailingInstructions => write!(f,"A code expression has invalid trailing instructions following its `end` instruction"),
            ValidationError::FunctionAndCodeSectionsHaveDifferentLengths => write!(f,"The function and code sections have different lengths"),
            ValidationError::DataCountAndDataSectionsLengthAreDifferent => write!(f,"The data count section specifies a different length than there are data segments in the data section"),
            ValidationError::InvalidImportType => f.write_str("Invalid import type"),
            ValidationError::InvalidStartFunctionSignature => write!(f,"The start function has parameters or return types which it is not allowed to have"),
            ValidationError::MissingDataCountSection => f.write_str("Some instructions could not be validated because the data count section is missing"),
            ValidationError::InvalidDataSegmentMode(mode) => write!(f, "The mode of a data segment was invalid (only 0..=2 is allowed): {mode}"),
            ValidationError::InvalidElementMode(mode) => write!(f, "The mode of an element was invalid (only 0..=7 is allowed): {mode}"),
            ValidationError::TooManyFunctions => f.write_str("The module contains too many functions. The maximum number of functions (either imported or locally-defined) is 2^32 - 1"),
            ValidationError::TooManyTables => f.write_str("The module contains too many tables. The maximum number of tables (either imported or locally-defined) is 2^32 - 1"),
            ValidationError::TooManyMemories => f.write_str("The module contains too many memories. The maximum number of memories (either imported or locally-defined) is 2^32 - 1"),
            ValidationError::TooManyGlobals => f.write_str("The module contains too many globals. The maximum number of globals (either imported or locally-defined) is 2^32 - 1"),
            ValidationError::LimitsMinLargerThanMax { min, max } => write!(f, "Limits are invalid because min={min} is larger than max={max}"),
            ValidationError::LimitsNotWithinRange(range) => write!(f, "The min or max field of a limits type is not within the expected range of {range}"),
            ValidationError::MutGlobalInConstGlobalGet => f.write_str("A mutable global was referenced in some global.get instruction in a constant expression"),
        }
    }
}

impl From<DecodingError> for ValidationError {
    fn from(error: DecodingError) -> Self {
        Self::Decoding(error)
    }
}

#[cfg(test)]
mod test {
    use alloc::string::ToString;

    use crate::core::decoding::error::DecodingError;

    #[test]
    fn fmt_invalid_magic() {
        assert!(DecodingError::InvalidMagic
            .to_string()
            .contains("magic number"));
    }

    #[test]
    fn fmt_invalid_version() {
        assert!(DecodingError::InvalidBinaryFormatVersion
            .to_string()
            .contains("version"));
    }
}
