use alloc::vec::Vec;

use const_interpreter_loop::run_const_span;
use function_ref::FunctionRef;
use interpreter_loop::run;
use value_stack::Stack;

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
pub(crate) mod locals;
pub mod registry;
pub mod store;
pub mod value;
pub mod value_stack;

/// The default module name if a [RuntimeInstance] was created using [RuntimeInstance::new].
pub const DEFAULT_MODULE: &str = "__interpreter_default__";

#[derive(Debug)]
pub struct RuntimeInstance<'b, H = EmptyHookSet>
where
    H: HookSet + core::fmt::Debug,
{
    pub hook_set: H,
    pub store: Store<'b>,
}

impl<'b> RuntimeInstance<'b, EmptyHookSet> {
    pub fn new(validation_info: &'_ ValidationInfo<'b>) -> CustomResult<Self> {
        Self::new_with_hooks(DEFAULT_MODULE, validation_info, EmptyHookSet)
    }

    pub fn new_named(
        module_name: &str,
        validation_info: &'_ ValidationInfo<'b>,
        // store: &mut Store,
    ) -> CustomResult<Self> {
        Self::new_with_hooks(module_name, validation_info, EmptyHookSet)
    }
}

impl<'b, H> RuntimeInstance<'b, H>
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

    pub fn new_with_hooks(
        module_name: &str,
        validation_info: &'_ ValidationInfo<'b>,
        hook_set: H,
        // store: &mut Store,
    ) -> CustomResult<Self> {
        trace!("Starting instantiation of bytecode");

        let store = Store::default();

        let mut instance = RuntimeInstance { hook_set, store };
        instance.add_module(module_name, validation_info)?;

        Ok(instance)
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
    pub fn invoke_typed<Param: InteropValueList, Returns: InteropValueList>(
        &mut self,
        function_ref: &FunctionRef,
        params: Param,
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
}
