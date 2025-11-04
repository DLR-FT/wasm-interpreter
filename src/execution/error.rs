use core::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RuntimeError {
    Trap(TrapError),

    ModuleNotFound,
    FunctionNotFound,
    ResumableNotFound,
    StackExhaustion,
    HostFunctionSignatureMismatch,
    WriteOnImmutableGlobal,
    GlobalTypeMismatch,
    /// A host function returned the [`HaltExecutionError`](crate::HaltExecutionError), which caused execution to be halted.
    HostFunctionHaltedExecution,
    // An access to a table element was out of bounds
    TableAccessOutOfBounds,
    UnknownExport,
    TableTypeMismatch,

    // Are all of these instantiation variants? Add a new `InstantiationError` enum?
    InvalidImportType,
    UnknownImport,
    /// It was attempted to register a symbol under a name for which a symbol already exists.
    RegistrySymbolAlreadyExists,
    MoreThanOneMemory,
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            RuntimeError::Trap(trap_error) => write!(f, "{trap_error}"),
            RuntimeError::FunctionNotFound => f.write_str("Function not found"),
            RuntimeError::ModuleNotFound => f.write_str("No such module exists"),
            RuntimeError::ResumableNotFound => f.write_str("No such resumable exists"),
            RuntimeError::StackExhaustion => {
                f.write_str("either the call stack or the value stack overflowed")
            }
            RuntimeError::HostFunctionSignatureMismatch => {
                f.write_str("host function call did not respect its type signature")
            }
            RuntimeError::HostFunctionHaltedExecution => {
                f.write_str("A host function requested execution to be halted.")
            }
            RuntimeError::InvalidImportType => f.write_str("Invalid import type"),
            RuntimeError::TableAccessOutOfBounds => f.write_str("A table access was out of bounds"),
            RuntimeError::RegistrySymbolAlreadyExists => f.write_str(
                "It was attempted to register a symbol under a name for which a symbol already exists.",
            ),
            RuntimeError::UnknownExport => {
                f.write_str("An unknown export was referenced by its name.")
            }
            RuntimeError::TableTypeMismatch => {
                f.write_str("An alloc/write operation failed on a table due to a type mismatch.")
            }
            // TODO: maybe move these to LinkerError also add more info to them (the name's export, function idx, etc)
            RuntimeError::UnknownImport => f.write_str("Unknown Import"),
            RuntimeError::MoreThanOneMemory => {
                f.write_str("As of not only one memory is allowed per module.")
            }

            RuntimeError::WriteOnImmutableGlobal => f.write_str(
                "A write operation on a global failed due to the global being immutable",
            ),
            RuntimeError::GlobalTypeMismatch => {
                f.write_str("An alloc/write operation on a global failed due to a type mismatch")
            }
        }
    }
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

impl From<TrapError> for RuntimeError {
    fn from(value: TrapError) -> Self {
        Self::Trap(value)
    }
}
