use wasm::{RefType, RuntimeError, TrapError, ValidationError};

pub fn match_runtime_error(actual: &RuntimeError, expected_message: &str) -> bool {
    runtime_error_to_message(actual).is_some_and(|actual| {
        actual.contains(expected_message)
            || (expected_message.contains("uninitialized element 2")
                && actual.contains("uninitialized element"))
    })
}

pub fn match_validation_error(actual: &ValidationError, expected_message: &str) -> bool {
    validation_error_to_message(actual)
        .is_some_and(|actual_message| actual_message.contains(expected_message))
}

fn trap_error_to_message(trap_error: &TrapError) -> Option<String> {
    use TrapError::*;
    let message = match trap_error {
        DivideBy0 => "integer divide by zero",
        UnrepresentableResult => "integer overflow",
        BadConversionToInteger => "invalid conversion to integer",
        ReachedUnreachable => "unreachable",
        MemoryOrDataAccessOutOfBounds => "out of bounds memory access",
        TableOrElementAccessOutOfBounds => "out of bounds table access",
        UninitializedElement => "uninitialized element",
        SignatureMismatch => "indirect call type mismatch",
        TableAccessOutOfBounds => "undefined element",
        IndirectCallNullFuncRef => return None,
    };
    Some(message.to_owned())
}

fn runtime_error_to_message(runtime_error: &RuntimeError) -> Option<String> {
    use RuntimeError::*;
    let message = match runtime_error {
        Trap(trap_error) => return trap_error_to_message(trap_error),
        StackExhaustion => "call stack exhausted",
        ModuleNotFound => "module not found",
        HostFunctionSignatureMismatch => "host function signature mismatch",
        _ => return None,
    };
    Some(message.to_owned())
}

fn validation_error_to_message(validation_error: &ValidationError) -> Option<String> {
    use ValidationError::*;
    let message = match validation_error {
        InvalidMagic => "magic header not detected",
        InvalidBinaryFormatVersion => "unknown binary version",
        Eof => "unexpected end of section or function",
        MalformedUtf8(_) => "malformed UTF-8 encoding",
        MalformedSectionTypeDiscriminator(_) => "malformed section id",
        MalformedNumTypeDiscriminator(_) => "malformed number type",
        MalformedVecTypeDiscriminator(_) => "malformed vector type",
        MalformedFuncTypeDiscriminator(_) => "malformed function type",
        // Note: MalformedValType does not exist in the reference
        // interpreter. Because it uses `either`, malformed value types are
        // recognized as malformed reference types instead.
        MalformedRefTypeDiscriminator(_) | ValidationError::MalformedValType => {
            "malformed reference type"
        }
        // TODO Not sure about this, maybe merge into above match arm. This also isn't tested for.
        I33IsNegative => "malformed reference type",
        MalformedExportDescDiscriminator(_) => "malformed export kind",
        MalformedImportDescDiscriminator(_) => "malformed import kind",
        MalformedMutDiscriminator(_) => "malformed mutability",
        VariableLengthIntegerRepresentationTooLong => "integer representation too long",

        VariableLengthIntegerOverflowed => "integer too large",
        MalformedLimitsDiscriminator(invalid_descriptor) => {
            // The spec interpreter incorrectly reads the limits descriptor as a
            // variable-lengh integer. Therefore, we have to distinguish between
            // its two error cases:
            if *invalid_descriptor > 0x80 {
                "integer representation too long"
            } else {
                "integer too large"
            }
        }
        MalformedElemKindDiscriminator(_) => "malformed element kind",
        MalformedMemArgFlags => "malformed memop flags",
        MalformedSpan => "length out of bounds",
        MemArgOffsetOverflowed => "i32 constant",
        SectionSizeMismatch => "section size mismatch",
        InvalidTypeIdx(type_idx) => return Some(format!("unknown type {type_idx}")),
        InvalidFuncIdx(func_idx) => return Some(format!("unknown function {func_idx}")),
        InvalidTableIdx(table_idx) => return Some(format!("unknown table {table_idx}")),
        InvalidMemIdx(mem_idx) => return Some(format!("unknown memory {mem_idx}")),
        InvalidGlobalIdx(global_idx) => return Some(format!("unknown global {global_idx}")),
        InvalidElemIdx(elem_idx) => return Some(format!("unknown elem segment {elem_idx}")),
        InvalidDataIdx(data_idx) => return Some(format!("unknown data segment {data_idx}")),
        InvalidLocalIdx(local_idx) => return Some(format!("unknown local {local_idx}")),
        InvalidLabelIdx(label_idx) => return Some(format!("unknown label {label_idx}")),
        InvalidLaneIdx(_) => "invalid lane index",
        InvalidLimitsMinLargerThanMax { .. } => "size minimum must not be greater than maximum",
        UnexpectedContentAfterLastSection => "unexpected content after last section",
        ExprMissingEnd => "unexpected end of section or function",
        InvalidSelectTypeVectorLength(_) => "invalid result arity",
        InvalidInstr(byte) => return Some(format!("illegal opcode {byte:02x}")),
        InvalidConstInstr(_) | InvalidConstMultiByteInstr(_, _) => "constant expression required",
        InvalidMultiByteInstr(byte, i) => {
            return Some(format!("illegal opcode {byte:02x} {i:02x}"))
        }
        EndInvalidValueStack
        | InvalidValidationStackValType(_)
        | ExpectedAnOperand
        | MismatchedRefTypesOnValidationStack { .. }
        | IfWithoutMatchingElse
        | ExpectedReferenceTypeOnStack(_) => "type mismatch",
        MemoryTooLarge => "memory size must be at most 65536 pages (4GiB)",
        MutationOfConstGlobal => "global is immutable",
        ErroneousAlignment { .. } => "alignment must not be larger than natural",
        ValidationCtrlStackEmpty => return None,
        // TODO check if this
        ElseWithoutMatchingIf => "misplaced ELSE opcode",
        MismatchedRefTypesDuringTableInit { elem_ty, table_ty } => {
            return Some(format!(
                "type mismatch: element segment's type {} does not match table's element type {}",
                ref_to_str(*elem_ty),
                ref_to_str(*table_ty)
            ));
        }
        MismatchedRefTypesDuringTableCopy {
            source_table_ty,
            destination_table_ty,
        } => {
            return Some(format!(
                "type mismatch: source element type {} does not match destination element type {}",
                ref_to_str(*source_table_ty),
                ref_to_str(*destination_table_ty)
            ))
        }
        IndirectCallToNonFuncRefTable(table_type) => {
            return Some(format!(
                "type mismatch: instruction requires table of functions but table has {}",
                ref_to_str(*table_type)
            ));
        }
        ReferencingAnUnreferencedFunction(_) => "undeclared function reference",
        TooManyLocals(_) => "too many locals",
        DuplicateExportName => "duplicate export name",
        UnsupportedMultipleMemoriesProposal => "multiple memories are not allowed (yet)",
        ExpectedZeroByte => "zero byte expected",
        CodeExprOverflow => "END opcode expected",

        CodeExprHasTrailingInstructions | LastCodeExprOverflow | InvalidCustomSectionLength => {
            "section size mismatch"
        }
        FunctionAndCodeSectionsHaveDifferentLengths => {
            "function and code section have inconsistent lengths"
        }
        DataCountAndDataSectionsLengthAreDifferent => {
            "data count and data section have inconsistent lengths"
        }
        InvalidStartFunctionSignature => "start function",
        ActiveElementSegmentTypeMismatch {
            active_element_type,
            table_ref_type,
        } => {
            return Some(format!(
                "type mismatch: element segment's type {} does not match table's element type {}",
                ref_to_str(*active_element_type),
                ref_to_str(*table_ref_type)
            ));
        }
        MissingDataCountSection => "data count section required",
        InvalidDataSegmentMode(_) => "malformed data segement kind",
        InvalidElementMode(_) => "malformed elements segment kind",
        TooManyFunctions | TooManyTables | TooManyMemories | TooManyGlobals => return None,
    };
    Some(message.to_owned())
}

fn ref_to_str(r: RefType) -> &'static str {
    match r {
        RefType::FuncRef => "funcref",
        RefType::ExternRef => "externref",
    }
}
