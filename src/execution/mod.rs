use alloc::vec::Vec;

use const_interpreter_loop::run_const_span;
use store::addrs::ModuleAddr;
use store::HaltExecutionError;
use value_stack::Stack;

use crate::execution::assert_validated::UnwrapValidatedExt;
use crate::execution::config::Config;
use crate::execution::store::Store;
use crate::execution::value::Value;
use crate::interop::InteropValueList;
use crate::{RuntimeError, ValidationInfo};

pub(crate) mod assert_validated;
pub mod config;
pub mod const_interpreter_loop;
pub mod error;
pub mod interop;
mod interpreter_loop;
pub mod linker;
pub(crate) mod little_endian;
pub mod registry;
pub mod resumable;
pub mod store;
pub mod value;
pub mod value_stack;

/// The default module name if a [RuntimeInstance] was created using [RuntimeInstance::new].
pub const DEFAULT_MODULE: &str = "__interpreter_default__";

pub struct RuntimeInstance<'b, T: Config = ()> {
    pub store: Store<'b, T>,
}

impl<T: Config + Default> Default for RuntimeInstance<'_, T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<'b, T: Config> RuntimeInstance<'b, T> {
    pub fn new(user_data: T) -> Self {
        RuntimeInstance {
            store: Store::new(user_data),
        }
    }

    // Returns the new [`RuntimeInstance`] and module addr of the default module.
    pub fn new_with_default_module(
        user_data: T,
        validation_info: &'_ ValidationInfo<'b>,
    ) -> Result<(Self, ModuleAddr), RuntimeError> {
        let mut instance = Self::new(user_data);
        let module_addr = instance
            .store
            .module_instantiate(validation_info, Vec::new(), None)?
            .module_addr;
        Ok((instance, module_addr))
    }

    // Returns the new [`RuntimeInstance`] and module addr of the new named module.
    pub fn new_named(
        user_data: T,
        _module_name: &str,
        validation_info: &'_ ValidationInfo<'b>,
        // store: &mut Store,
    ) -> Result<(Self, ModuleAddr), RuntimeError> {
        let mut instance = Self::new(user_data);
        let module_addr = instance
            .store
            .module_instantiate(validation_info, Vec::new(), None)?
            .module_addr;
        Ok((instance, module_addr))
    }
}

/// Helper function to quickly construct host functions without worrying about wasm to Rust
/// type conversion. For reading/writing user data into the current configuration, simply move
/// `user_data` into the passed closure.
/// # Example
/// ```
/// use wasm::{validate, RuntimeInstance, host_function_wrapper, Value, HaltExecutionError};
/// fn my_wrapped_host_func(user_data: &mut (), params: Vec<Value>) -> Result<Vec<Value>, HaltExecutionError> {
///     host_function_wrapper(params, |(x, y): (u32, i32)| -> Result<u32, HaltExecutionError> {
///         let _user_data = user_data;
///         Ok(x + (y as u32))
///     })
/// }
/// fn main() {
///     let mut instance = RuntimeInstance::new(());
///     let foo_bar = instance.store.func_alloc_typed::<(u32, i32), u32>(my_wrapped_host_func);
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
