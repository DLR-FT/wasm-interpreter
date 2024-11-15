#![no_std]
extern crate alloc;
#[macro_use]
extern crate log;

pub use core::error::{Error, Result, RuntimeError};
pub use core::reader::types::{Limits, NumType, RefType, ValType};
pub use execution::value::Value;
pub use execution::*;
pub use validation::*;

pub(crate) mod core;
pub(crate) mod execution;
pub(crate) mod validation;
