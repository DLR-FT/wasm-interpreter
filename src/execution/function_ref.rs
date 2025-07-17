use alloc::borrow::ToOwned;
use alloc::vec::Vec;

use crate::execution::{hooks::HookSet, value::InteropValueList, RuntimeInstance};
use crate::{Error, ExternVal, Result as CustomResult, RuntimeError, Store, ValType, Value};

pub struct FunctionRef {
    pub func_addr: usize,
}

impl FunctionRef {
    pub fn new_from_name(
        module_name: &str,
        function_name: &str,
        store: &Store,
    ) -> CustomResult<Self> {
        // https://webassembly.github.io/spec/core/appendix/embedding.html#module-instances
        // inspired by instance_export
        let extern_val = store
            .registry
            .lookup(
                module_name.to_owned().into(),
                function_name.to_owned().into(),
            )
            .map_err(|_| Error::RuntimeError(RuntimeError::FunctionNotFound))?;
        match extern_val {
            ExternVal::Func(func_addr) => Ok(Self {
                func_addr: *func_addr,
            }),
            _ => Err(Error::RuntimeError(RuntimeError::FunctionNotFound)),
        }
    }

    pub fn invoke<
        H: HookSet + core::fmt::Debug,
        Param: InteropValueList,
        Returns: InteropValueList,
    >(
        &self,
        runtime: &mut RuntimeInstance<H>,
        params: Param,
        // store: &mut Store,
    ) -> Result<Returns, RuntimeError> {
        runtime.invoke(self, params /* , store */)
    }

    pub fn invoke_dynamic<H: HookSet + core::fmt::Debug>(
        &self,
        runtime: &mut RuntimeInstance<H>,
        params: Vec<Value>,
        ret_types: &[ValType],
        // store: &mut Store,
    ) -> Result<Vec<Value>, RuntimeError> {
        runtime.invoke_dynamic(self, params, ret_types /* , store */)
    }
}
