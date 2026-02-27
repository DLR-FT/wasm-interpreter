use const_interpreter_loop::run_const_span;
use value_stack::Stack;

use crate::execution::assert_validated::UnwrapValidatedExt;

pub(crate) mod assert_validated;
pub mod config;
pub mod const_interpreter_loop;
pub mod error;
mod interpreter_loop;
pub(crate) mod little_endian;
pub mod resumable;
pub mod store;
pub mod value;
pub mod value_stack;