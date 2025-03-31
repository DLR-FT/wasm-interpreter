use alloc::string::String;
use alloc::vec::Vec;

use crate::execution::{hooks::HookSet, value::InteropValueList, RuntimeInstance};
use crate::{RuntimeError, ValType, Value};

pub struct FunctionRef {
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
    pub fn invoke<H: HookSet, Param: InteropValueList, Returns: InteropValueList>(
        &self,
        runtime: &mut RuntimeInstance<H>,
        params: Param,
    ) -> Result<Returns, RuntimeError> {
        if runtime.lut.is_none() {
            panic!("At the disco");
        }
        todo!();
    }

    pub fn invoke_dynamic<H: HookSet>(
        &self,
        runtime: &mut RuntimeInstance<H>,
        params: Vec<Value>,
        ret_types: &[ValType],
    ) -> Result<Vec<Value>, RuntimeError> {
        if runtime.lut.is_none() {
            panic!("At the disco");
        }

        runtime.invoke_dynamic(self, params, ret_types)
    }

    // pub fn get_return_types(&self) -> Vec<Value
}
