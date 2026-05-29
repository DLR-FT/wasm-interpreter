//! An in-place interpreter for WebAssembly 2.0
//!
//! # General Usage
//!
//! WebAssembly (Wasm) modules must first be decoded and validated through [`decode_and_validate`],
//! producing a [`Module`]. This module can then be instantiated in a [`Store`] via
//! [`Store::module_instantiate`], creating a module instance and returning its module address,
//! uniquely identifying this module instance within that store.
//!
//! When a [`Store`] is initially created through [`Store::new`], it is empty. This store exposes
//! many other functions besides module instantiation to interact with it and objects allocated
//! within it. Most notably, the [`Store::invoke_simple`] and [`Store::invoke`] methods are used to
//! interpret Wasm code. Refer to the examples in `examples/` for more information.
//!
//! # Example
//!
//! This is an example for how to run a function exposed from a simple Wasm module (full code in
//! `examples/function_invocation.rs`):
//!
//! ```
//! # use dlr_wasm_interpreter::{ExternVal, FuncAddr, InstantiationOutcome, Module, Store, Value, decode_and_validate};
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! const WAT_CODE: &str = r#"
//! (module
//!     (func (export "add_one") (param $n i32) (result i32)
//!         local.get $n
//!         i32.const 1
//!         i32.add))
//! "#;
//!
//! // Use the `wat` crate to convert from the text format to bytecode
//! let wasm_bytecode = wat::parse_str(WAT_CODE)?;
//!
//! // Decode and validate the module
//! let module = decode_and_validate(&wasm_bytecode, &mut ())?;
//!
//! // Create a new empty store
//! let mut store = Store::new(());
//!
//! // Instantiate the module to create a module instance, returning its address
//! // SAFETY: There are no extern values.
//! let module_addr = unsafe { store.module_instantiate(&module, vec![], None) }?.module_addr;
//!
//! // Get the function address of the exported add_one function
//! // SAFETY: The module address was returned from the same store.
//! let add_one_extern = unsafe { store.instance_export(module_addr, "add_one") }?;
//! let add_one = add_one_extern.as_func().ok_or("add_one is not a function")?;
//!
//! // Invoke the function
//! // SAFETY: The function address was returned from the same store. There are also no address
//! // type parameters.
//! let return_values = unsafe { store.invoke_simple(add_one, vec![Value::I32(16)]) }?;
//! assert_eq!(*return_values, [Value::I32(17)]);
//! # Ok(())
//! # }
//! ```

#![no_std]

extern crate alloc;

pub use crate::{
    core::{
        decoding::{error::DecodingError, modules::custom_section::CustomSection},
        rw_spinlock,
        structure::instructions,
        structure::types::{
            ExternType, FuncType, GlobalType, Limits, MemType, NumType, RefType, ResultType,
            TableType, ValType,
        },
    },
    execution::{
        config::Config,
        error::{RuntimeError, TrapError},
        resumable::*,
        runtime_structure::{
            addresses::*,
            external_values::ExternVal,
            memory_instances::shared_linear_memory::{Ordering, SharedLinearMemory},
            store::{Hostcode, InstantiationOutcome, Store},
            values::{ExternAddr, Ref, Value, ValueTypeMismatchError, F32, F64},
        },
    },
    validation::{config::ValidationConfig, decode_and_validate, error::ValidationError, Module},
};

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
    use crate::{core::decoding::error::DecodingError, Error, RuntimeError, ValidationError};

    #[test]
    fn error_conversion_validation_error() {
        let validation_error = ValidationError::Decoding(DecodingError::InvalidMagic);
        let error: Error = validation_error.into();

        assert_eq!(
            error,
            Error::Validation(ValidationError::Decoding(DecodingError::InvalidMagic))
        )
    }

    #[test]
    fn error_conversion_runtime_error() {
        let runtime_error = RuntimeError::ModuleNotFound;
        let error: Error = runtime_error.into();

        assert_eq!(error, Error::RuntimeError(RuntimeError::ModuleNotFound))
    }
}
