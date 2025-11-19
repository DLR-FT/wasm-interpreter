#![no_std]
#![deny(clippy::undocumented_unsafe_blocks)]

extern crate alloc;
#[macro_use]
extern crate log_wrapper;

pub use core::error::ValidationError;
pub use core::reader::types::{
    export::ExportDesc, global::GlobalType, Limits, MemType, NumType, RefType, TableType, ValType,
};
pub use core::rw_spinlock;
pub use execution::error::{RuntimeError, TrapError};

pub use execution::store::*;
pub use execution::value::Value;
pub use execution::*;
pub use validation::*;

pub(crate) mod core;
pub(crate) mod execution;
pub(crate) mod validation;

/// A definition for a [`Result`] using the optional [`Error`] type.
pub type Result<T> = ::core::result::Result<T, Error>;

/// An opt-in error type useful for merging all error types of this crate into a single type.
///
/// Note: This crate does not use this type in any public interfaces, making it optional for downstream users.
#[derive(Debug, PartialEq, Eq)]
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

#[cfg(test)]
mod test {
    use crate::{Error, RuntimeError, ValidationError};

    #[test]
    fn error_conversion_validation_error() {
        let validation_error = ValidationError::InvalidMagic;
        let error: Error = validation_error.into();

        assert_eq!(error, Error::Validation(ValidationError::InvalidMagic))
    }

    #[test]
    fn error_conversion_runtime_error() {
        let runtime_error = RuntimeError::ModuleNotFound;
        let error: Error = runtime_error.into();

        assert_eq!(error, Error::RuntimeError(RuntimeError::ModuleNotFound))
    }
}
