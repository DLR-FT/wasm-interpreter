use alloc::vec::Vec;

use crate::execution::{hooks::HookSet, value::InteropValueList, RuntimeInstance};
use crate::{
    Error, ExportInst, ExternVal, Result as CustomResult, RuntimeError, Store, ValType, Value,
};

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
        let module_addr = *store
            .module_names
            .get(module_name)
            .ok_or(Error::RuntimeError(RuntimeError::ModuleNotFound))?;
        Ok(Self {
            func_addr: *&store.modules[module_addr]
                .exports
                .iter()
                .find_map(|ExportInst { name, value }| {
                    if *name == function_name {
                        match value {
                            ExternVal::Func(func_addr) => Some(*func_addr),
                            _ => None,
                        }
                    } else {
                        None
                    }
                })
                .ok_or(Error::RuntimeError(RuntimeError::FunctionNotFound))?,
        })
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
