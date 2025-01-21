use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::vec::Vec;

use crate::execution::{hooks::HookSet, value::InteropValueList, RuntimeInstance};
use crate::{Result as CustomResult, RuntimeError, Store, ValType, Value};

pub struct FunctionRef {
    // inner: InnerFunctionRef,
    pub(crate) module_name: String,
    pub(crate) function_name: String,
    pub(crate) module_index: usize,
    pub(crate) function_index: usize,
    /// If the function is exported from the module or not. This is used to determine if the function name - index
    /// mapping should be verified. The module name - index mapping is always verified.
    ///
    /// If this is set to false then the user must make sure that the function reference will still be valid when the
    /// function is called. This means that the module must not be unloaded.
    pub(crate) exported: bool,
}

impl FunctionRef {
    pub fn new_from_name(
        module_name: &str,
        function_name: &str,
        store: &Store,
    ) -> CustomResult<Self> {
        let module_idx = store.get_module_idx_from_name(module_name)?;
        let func_idx = store
            .get_local_function_idx_by_function_name(module_idx, function_name)
            .ok_or(RuntimeError::FunctionNotFound)?;

        Ok(Self {
            function_name: function_name.to_owned(),
            module_name: module_name.to_owned(),
            function_index: func_idx,
            module_index: module_idx,
            exported: false,
        })
    }

    pub fn invoke<H: HookSet, Param: InteropValueList, Returns: InteropValueList>(
        &self,
        runtime: &mut RuntimeInstance<H>,
        params: Param,
        // store: &mut Store,
    ) -> Result<Returns, RuntimeError> {
        runtime.invoke(self, params /* , store */)
    }

    pub fn invoke_dynamic<H: HookSet>(
        &self,
        runtime: &mut RuntimeInstance<H>,
        params: Vec<Value>,
        ret_types: &[ValType],
        // store: &mut Store,
    ) -> Result<Vec<Value>, RuntimeError> {
        runtime.invoke_dynamic(self, params, ret_types /* , store */)
    }
}
