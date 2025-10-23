use alloc::borrow::ToOwned;
use alloc::vec::Vec;

use crate::config::Config;
use crate::execution::{hooks::HookSet, interop::InteropValueList, RuntimeInstance};
use crate::{ExternVal, RuntimeError, Store, Value};

pub struct FunctionRef {
    pub func_addr: usize,
}

impl FunctionRef {
    pub fn new_from_name<T, C: Config>(
        module_name: &str,
        function_name: &str,
        store: &Store<T, C>,
    ) -> Result<Self, RuntimeError> {
        // https://webassembly.github.io/spec/core/appendix/embedding.html#module-instances
        // inspired by instance_export
        let extern_val = store
            .registry
            .lookup(
                module_name.to_owned().into(),
                function_name.to_owned().into(),
            )
            .map_err(|_| RuntimeError::FunctionNotFound)?;
        match extern_val {
            ExternVal::Func(func_addr) => Ok(Self {
                func_addr: *func_addr,
            }),
            _ => Err(RuntimeError::FunctionNotFound),
        }
    }

    pub fn invoke_typed<
        H: HookSet + core::fmt::Debug,
        Param: InteropValueList,
        Returns: InteropValueList,
    >(
        &self,
        runtime: &mut RuntimeInstance<H>,
        params: Param,
        // store: &mut Store,
    ) -> Result<Returns, RuntimeError> {
        runtime.invoke_typed(self, params /* , store */)
    }

    pub fn invoke<T, H: HookSet + core::fmt::Debug>(
        &self,
        runtime: &mut RuntimeInstance<T, H>,
        params: Vec<Value>,
        // store: &mut Store,
    ) -> Result<Vec<Value>, RuntimeError> {
        runtime.invoke(self, params)
    }
}
