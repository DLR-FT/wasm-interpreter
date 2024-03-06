#![no_std]
extern crate alloc;
#[macro_use]
extern crate log;

pub use core::error::{Error, Result};
pub use validation::{validate, ValidationInfo};

pub use execution::{instantiate, invocate_fn, InstantiatedInstance, value};

pub(crate) mod core;
pub(crate) mod execution;
pub(crate) mod validation;
