use alloc::vec::Vec;

use const_interpreter_loop::run_const_span;
use store::HaltExecutionError;
use value_stack::Stack;

use crate::execution::assert_validated::UnwrapValidatedExt;
use crate::execution::value::Value;
use crate::interop::InteropValueList;

pub(crate) mod assert_validated;
pub mod checked;
pub mod config;
pub mod const_interpreter_loop;
pub mod error;
pub mod interop;
mod interpreter_loop;
pub mod linker;
pub(crate) mod little_endian;
pub mod resumable;
pub mod store;
pub mod value;
pub mod value_stack;

/// Helper function to quickly construct host functions without worrying about wasm to Rust
/// type conversion. For reading/writing user data into the current configuration, simply move
/// `user_data` into the passed closure.
/// # Example
/// ```
/// use wasm::{validate,  Store, host_function_wrapper, Value, HaltExecutionError};
/// fn my_wrapped_host_func(user_data: &mut Store<()>, params: Vec<Value>) -> Result<Vec<Value>, HaltExecutionError> {
///     host_function_wrapper(params, |(x, y): (u32, i32)| -> Result<u32, HaltExecutionError> {
///         let _user_data = user_data;
///         Ok(x + (y as u32))
///     })
/// }
/// fn main() {
///     let mut store = Store::new(());
///     let foo_bar = store.func_alloc_typed_unchecked::<(u32, i32), u32>(my_wrapped_host_func);
/// }
/// ```
pub fn host_function_wrapper<Params: InteropValueList, Results: InteropValueList>(
    params: Vec<Value>,
    f: impl FnOnce(Params) -> Result<Results, HaltExecutionError>,
) -> Result<Vec<Value>, HaltExecutionError> {
    let params =
        Params::try_from_values(params.into_iter()).expect("Params match the actual parameters");
    f(params).map(Results::into_values)
}
