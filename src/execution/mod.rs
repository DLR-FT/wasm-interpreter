use crate::Error;

use alloc::borrow::ToOwned;
use alloc::vec::Vec;

use const_interpreter_loop::run_const_span;
use function_ref::FunctionRef;
use value_stack::Stack;

use crate::core::reader::types::{FuncType, ResultType};
use crate::execution::assert_validated::UnwrapValidatedExt;
use crate::execution::hooks::{EmptyHookSet, HookSet};
use crate::execution::store::Store;
use crate::execution::value::Value;
use crate::value::InteropValueList;
use crate::{Result as CustomResult, RuntimeError, ValidationInfo};

pub(crate) mod assert_validated;
pub mod const_interpreter_loop;
pub mod function_ref;
pub mod hooks;
mod interpreter_loop;
pub(crate) mod linear_memory;
pub mod registry;
pub mod store;
pub mod value;
pub mod value_stack;

/// The default module name if a [RuntimeInstance] was created using [RuntimeInstance::new].
pub const DEFAULT_MODULE: &str = "__interpreter_default__";

#[derive(Debug)]
pub struct RuntimeInstance<'b, T = (), H = EmptyHookSet>
where
    H: HookSet + core::fmt::Debug,
{
    pub hook_set: H,
    pub store: Store<'b, T>,
}

impl<T: Default> Default for RuntimeInstance<'_, T, EmptyHookSet> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<'b, T> RuntimeInstance<'b, T, EmptyHookSet> {
    pub fn new(user_data: T) -> Self {
        Self::new_with_hooks(user_data, EmptyHookSet)
    }

    pub fn new_with_default_module(
        user_data: T,
        validation_info: &'_ ValidationInfo<'b>,
    ) -> CustomResult<Self> {
        let mut instance = Self::new_with_hooks(user_data, EmptyHookSet);
        instance.add_module(DEFAULT_MODULE, validation_info)?;
        Ok(instance)
    }

    pub fn new_named(
        user_data: T,
        module_name: &str,
        validation_info: &'_ ValidationInfo<'b>,
        // store: &mut Store,
    ) -> CustomResult<Self> {
        let mut instance = Self::new_with_hooks(user_data, EmptyHookSet);
        instance.add_module(module_name, validation_info)?;
        Ok(instance)
    }
}

impl<'b, T, H> RuntimeInstance<'b, T, H>
where
    H: HookSet + core::fmt::Debug,
{
    pub fn add_module(
        &mut self,
        module_name: &str,
        validation_info: &'_ ValidationInfo<'b>,
    ) -> CustomResult<()> {
        self.store.add_module(module_name, validation_info)
    }

    pub fn new_with_hooks(user_data: T, hook_set: H) -> Self {
        RuntimeInstance {
            hook_set,
            store: Store::new(user_data),
        }
    }

    pub fn get_function_by_name(
        &self,
        module_name: &str,
        function_name: &str,
    ) -> Result<FunctionRef, RuntimeError> {
        FunctionRef::new_from_name(module_name, function_name, &self.store)
            .map_err(|_| RuntimeError::FunctionNotFound)
    }

    pub fn get_function_by_index(
        &self,
        module_addr: usize,
        function_idx: usize,
    ) -> Result<FunctionRef, RuntimeError> {
        let module_inst = self
            .store
            .modules
            .get(module_addr)
            .ok_or(RuntimeError::ModuleNotFound)?;
        let func_addr = *module_inst
            .func_addrs
            .get(function_idx)
            .ok_or(RuntimeError::FunctionNotFound)?;

        Ok(FunctionRef { func_addr })
    }

    /// Invokes a function with the given parameters of type `Param`, and return types of type `Returns`.
    pub fn invoke_typed<Params: InteropValueList, Returns: InteropValueList>(
        &mut self,
        function_ref: &FunctionRef,
        params: Params,
        // store: &mut Store,
    ) -> Result<Returns, RuntimeError> {
        let FunctionRef { func_addr } = *function_ref;
        self.store
            .invoke(func_addr, params.into_values())
            .map(|values| Returns::from_values(values.into_iter()))
    }

    /// Invokes a function with the given parameters. The return types depend on the function signature.
    pub fn invoke(
        &mut self,
        function_ref: &FunctionRef,
        params: Vec<Value>,
    ) -> Result<Vec<Value>, RuntimeError> {
        let FunctionRef { func_addr } = *function_ref;
        self.store.invoke(func_addr, params)
    }

    /// Adds a host function under module namespace `module_name` with name `name`.
    /// roughly similar to `func_alloc` in <https://webassembly.github.io/spec/core/appendix/embedding.html#functions>
    /// except the host function is made visible to other modules through these names.
    pub fn add_host_function_typed<Params: InteropValueList, Returns: InteropValueList>(
        &mut self,
        module_name: &str,
        name: &str,
        host_func: fn(&mut T, Vec<Value>) -> Vec<Value>,
    ) -> Result<FunctionRef, Error> {
        let host_func_ty = FuncType {
            params: ResultType {
                valtypes: Vec::from(Params::TYS),
            },
            returns: ResultType {
                valtypes: Vec::from(Returns::TYS),
            },
        };
        self.add_host_function(module_name, name, host_func_ty, host_func)
    }

    pub fn add_host_function(
        &mut self,
        module_name: &str,
        name: &str,
        host_func_ty: FuncType,
        host_func: fn(&mut T, Vec<Value>) -> Vec<Value>,
    ) -> Result<FunctionRef, Error> {
        let func_addr = self.store.alloc_host_func(host_func_ty, host_func);
        self.store.registry.register(
            module_name.to_owned().into(),
            name.to_owned().into(),
            store::ExternVal::Func(func_addr),
        )?;
        Ok(FunctionRef { func_addr })
    }

    pub fn user_data(&self) -> &T {
        &self.store.user_data
    }

    pub fn user_data_mut(&mut self) -> &mut T {
        &mut self.store.user_data
    }
}

/// Helper function to quickly construct host functions without worrying about wasm to Rust
/// type conversion. For user data, simply move the mutable reference into the passed closure.
/// # Example
/// ```
/// use wasm::{validate, RuntimeInstance, host_function_wrapper, Value};
/// fn my_wrapped_host_func(user_data: &mut (), params: Vec<Value>) -> Vec<Value> {
///     host_function_wrapper(params, |(x, y): (u32, i32)| -> u32 {
///         let _user_data = user_data;
///         x + (y as u32)
///  })
/// }
/// fn main() {
///     let mut instance = RuntimeInstance::new(());
///     let foo_bar = instance.add_host_function_typed::<(u32,i32),u32>("foo", "bar", my_wrapped_host_func).unwrap();
/// }
/// ```
pub fn host_function_wrapper<Params: InteropValueList, Results: InteropValueList>(
    params: Vec<Value>,
    f: impl FnOnce(Params) -> Results,
) -> Vec<Value> {
    let params = Params::from_values(params.into_iter());
    let results = f(params);
    results.into_values()
}
