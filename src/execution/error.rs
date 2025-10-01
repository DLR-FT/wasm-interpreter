use core::fmt::{self, Display, Formatter};

pub enum RuntimeOrHostError<HostError> {
    Runtime(RuntimeError),
    Host(HostError),
}

impl<HostError: fmt::Debug> fmt::Debug for RuntimeOrHostError<HostError> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Runtime(runtime_error) => f.debug_tuple("Runtime").field(runtime_error).finish(),
            Self::Host(host_error) => f.debug_tuple("Host").field(host_error).finish(),
        }
    }
}

impl<HostError: fmt::Display> fmt::Display for RuntimeOrHostError<HostError> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeOrHostError::Runtime(runtime_error) => {
                write!(f, "Runtime error: {runtime_error}")
            }
            RuntimeOrHostError::Host(host_error) => write!(f, "Host error: {host_error}"),
        }
    }
}

impl<HostError: core::error::Error + 'static> core::error::Error for RuntimeOrHostError<HostError> {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            RuntimeOrHostError::Runtime(runtime_error) => Some(runtime_error),
            RuntimeOrHostError::Host(host_error) => Some(host_error),
        }
    }
}

impl<HostError: Clone> Clone for RuntimeOrHostError<HostError> {
    fn clone(&self) -> Self {
        match self {
            Self::Runtime(runtime_error) => Self::Runtime(runtime_error.clone()),
            Self::Host(host_error) => Self::Host(host_error.clone()),
        }
    }
}

impl<HostError: PartialEq> PartialEq for RuntimeOrHostError<HostError> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Runtime(a), Self::Runtime(b)) => a == b,
            (Self::Host(a), Self::Host(b)) => a == b,
            _ => false,
        }
    }
}

impl<HostError: Eq> Eq for RuntimeOrHostError<HostError> {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeError {
    Trap(TrapError),

    ModuleNotFound,
    FunctionNotFound,
    ResumableNotFound,
    StackExhaustion,
    HostFunctionSignatureMismatch,

    // Are all of these instantiation variants? Add a new `InstantiationError` enum?
    InvalidImportType,
    UnknownImport,
    MoreThanOneMemory,
    OutOfFuel,
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
            RuntimeError::InvalidImportType => f.write_str("Invalid import type"),
            // TODO: maybe move these to LinkerError also add more info to them (the name's export, function idx, etc)
            RuntimeError::UnknownImport => f.write_str("Unknown Import"),
            RuntimeError::MoreThanOneMemory => {
                f.write_str("As of not only one memory is allowed per module.")
            }
            RuntimeError::OutOfFuel => f.write_str("ran out of fuel"),
        }
    }
}

impl core::error::Error for RuntimeError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            RuntimeError::Trap(trap_err) => Some(trap_err),
            _ => None,
        }
    }
}

impl<HostError> From<RuntimeError> for RuntimeOrHostError<HostError> {
    fn from(value: RuntimeError) -> Self {
        Self::Runtime(value)
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

impl core::error::Error for TrapError {}

impl From<TrapError> for RuntimeError {
    fn from(value: TrapError) -> Self {
        Self::Trap(value)
    }
}

impl<HostError> From<TrapError> for RuntimeOrHostError<HostError> {
    fn from(value: TrapError) -> Self {
        Self::Runtime(RuntimeError::Trap(value))
    }
}
