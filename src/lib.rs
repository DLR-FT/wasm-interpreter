#![no_std]
#![deny(clippy::undocumented_unsafe_blocks)]

extern crate alloc;
#[macro_use]
extern crate log;

pub use core::error::{Error, Result, RuntimeError, TrapError};
pub use core::reader::types::{export::ExportDesc, Limits, NumType, RefType, ValType};
pub use core::rw_spinlock;
pub use execution::store::*;
pub use execution::value::Value;
pub use execution::*;
pub use validation::*;

pub(crate) mod core;
pub(crate) mod execution;
pub(crate) mod validation;
