use alloc::borrow::ToOwned;
use alloc::vec::Vec;

use crate::execution::{hooks::HookSet, interop::InteropValueList, RuntimeInstance};
use crate::{ExternVal, RuntimeError, Store, Value};

use super::error::RuntimeOrHostError;

pub struct FunctionRef {
    pub func_addr: usize,
}

impl FunctionRef {
    pub fn new_from_name<T, HostError: core::fmt::Debug>(
        module_name: &str,
        function_name: &str,
        store: &Store<T, HostError>,
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
        T,
        H: HookSet + core::fmt::Debug,
        HostError: core::fmt::Debug,
        Param: InteropValueList,
        Returns: InteropValueList,
    >(
        &self,
        runtime: &mut RuntimeInstance<T, H, HostError>,
        params: Param,
        // store: &mut Store,
    ) -> Result<Returns, RuntimeOrHostError<HostError>> {
        runtime.invoke_typed(self, params /* , store */)
    }

    pub fn invoke<T, H: HookSet + core::fmt::Debug, HostError: core::fmt::Debug>(
        &self,
        runtime: &mut RuntimeInstance<T, H, HostError>,
        params: Vec<Value>,
        // store: &mut Store,
    ) -> Result<Vec<Value>, RuntimeOrHostError<HostError>> {
        runtime.invoke(self, params)
    }
}
