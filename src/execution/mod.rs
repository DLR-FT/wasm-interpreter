use alloc::borrow::ToOwned;
use alloc::string::ToString;
use alloc::vec::Vec;

use const_interpreter_loop::{run_const, run_const_span};
use function_ref::FunctionRef;
use interpreter_loop::run;
use locals::Locals;
use value::Ref;
use value_stack::Stack;

use crate::core::reader::types::export::ExportDesc;
use crate::execution::assert_validated::UnwrapValidatedExt;
use crate::execution::hooks::{EmptyHookSet, HookSet};
use crate::execution::store::Store;
use crate::execution::value::Value;
use crate::value::InteropValueList;
use crate::{Result as CustomResult, RuntimeError, ValType, ValidationInfo};

pub(crate) mod assert_validated;
pub mod const_interpreter_loop;
pub(crate) mod execution_info;
pub mod function_ref;
pub mod hooks;
mod interpreter_loop;
pub(crate) mod linear_memory;
pub(crate) mod locals;
pub mod store;
pub mod value;
pub mod value_stack;

/// The default module name if a [RuntimeInstance] was created using [RuntimeInstance::new].
pub const DEFAULT_MODULE: &str = "__interpreter_default__";

pub struct RuntimeInstance<'b, H = EmptyHookSet>
where
    H: HookSet,
{
    pub hook_set: H,
    pub store: Option<Store<'b>>,
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
    H: HookSet,
{
    pub fn add_module(
        &mut self,
        module_name: &str,
        validation_info: &'_ ValidationInfo<'b>,
    ) -> CustomResult<()> {
        match self.store {
            None => return Err(crate::Error::RuntimeError(RuntimeError::StoreNotFound)),
            Some(ref mut store) => {
                store.add_module(module_name.to_owned(), validation_info.clone())?;
            }
        };
        Ok(())
    }

    pub fn new_with_hooks(
        module_name: &str,
        validation_info: &'_ ValidationInfo<'b>,
        hook_set: H,
        // store: &mut Store,
    ) -> CustomResult<Self> {
        trace!("Starting instantiation of bytecode");

        let store = Some(Store::default());

        let start = validation_info.start.map(|start| FunctionRef {
            module_name: module_name.to_string(),
            function_name: "start".to_string(),
            module_index: 0,
            function_index: start,
            exported: false,
        });

        let mut instance = RuntimeInstance { hook_set, store };
        instance.add_module(module_name, validation_info)?;

        if let Some(start_fn) = start {
            instance.invoke::<(), ()>(&start_fn, ())?;
        }

        Ok(instance)
    }

    pub fn get_function_by_name(
        &self,
        module_name: &str,
        function_name: &str,
    ) -> Result<FunctionRef, RuntimeError> {
        let (module_idx, func_idx) = self.get_indicies(module_name, function_name)?;

        Ok(FunctionRef {
            module_name: module_name.to_string(),
            function_name: function_name.to_string(),
            module_index: module_idx,
            function_index: func_idx,
            exported: true,
        })
    }

    pub fn get_function_by_index(
        &self,
        module_idx: usize,
        function_idx: usize,
    ) -> Result<FunctionRef, RuntimeError> {
        if self.store.is_none() {
            return Err(RuntimeError::StoreNotFound);
        }

        let module = (self.store)
            .as_ref()
            .unwrap_validated()
            .modules
            .get(module_idx)
            .ok_or(RuntimeError::ModuleNotFound)?;

        let function_name = module
            .exports
            .iter()
            .find(|export| match &export.desc {
                ExportDesc::FuncIdx(idx) => *idx == function_idx,
                _ => false,
            })
            .map(|export| export.name.clone())
            .ok_or(RuntimeError::FunctionNotFound)?;

        Ok(FunctionRef {
            module_name: module.name.clone(),
            function_name,
            module_index: module_idx,
            function_index: function_idx,
            exported: true,
        })
    }

    pub fn invoke<Param: InteropValueList, Returns: InteropValueList>(
        &mut self,
        function_ref: &FunctionRef,
        params: Param,
        // store: &mut Store,
    ) -> Result<Returns, RuntimeError> {
        if self.store.is_none() {
            return Err(RuntimeError::StoreNotFound);
        }
        // First, verify that the function reference is valid
        let (module_idx, func_idx) = self.verify_function_ref(function_ref)?;

        // -=-= Verification =-=-
        trace!(
            "Global function idx: {:?}",
            self.store.as_ref().unwrap_validated().modules[module_idx].functions[func_idx]
        );

        let func_inst_idx = *self.store.as_ref().unwrap_validated().modules[module_idx]
            .functions
            .get(func_idx)
            .ok_or(RuntimeError::FunctionNotFound)?;

        let func_inst = self
            .store
            .as_ref()
            .unwrap_validated()
            .functions
            .get(func_inst_idx)
            .ok_or(RuntimeError::FunctionNotFound)?;

        let func_ty = func_inst.ty();

        // Check correct function parameters and return types
        if func_ty.params.valtypes != Param::TYS {
            panic!("Invalid `Param` generics");
        }
        if func_ty.returns.valtypes != Returns::TYS {
            panic!("Invalid `Returns` generics");
        }

        // Prepare a new stack with the locals for the entry function
        let mut stack = Stack::new();
        let locals = Locals::new(
            params.into_values().into_iter(),
            func_inst.locals.iter().cloned(),
        );

        // setting `usize::MAX` as return address for the outermost function ensures that we
        // observably fail upon errornoeusly continuing execution after that function returns.
        stack.push_stackframe(
            module_idx,
            func_idx,
            &func_ty,
            locals,
            usize::MAX,
            usize::MAX,
        );

        let mut current_module_idx = module_idx;

        // Run the interpreter
        run(
            &mut current_module_idx,
            &mut stack,
            EmptyHookSet,
            self.store.as_mut().unwrap_validated(),
        )?;

        // Pop return values from stack
        let return_values = Returns::TYS
            .iter()
            .rev()
            .map(|ty| stack.pop_value(*ty))
            .collect::<Vec<Value>>();

        // Values are reversed because they were popped from stack one-by-one. Now reverse them back
        let reversed_values = return_values.into_iter().rev();
        let ret: Returns = Returns::from_values(reversed_values);
        debug!("Successfully invoked function");
        Ok(ret)
    }

    /// Invokes a function with the given parameters, and return types which are not known at compile time.
    pub fn invoke_dynamic(
        &mut self,
        function_ref: &FunctionRef,
        params: Vec<Value>,
        ret_types: &[ValType],
        // store: &mut Store,
    ) -> Result<Vec<Value>, RuntimeError> {
        if self.store.is_none() {
            return Err(RuntimeError::StoreNotFound);
        }
        // First, verify that the function reference is valid
        let (module_idx, func_idx) = self.verify_function_ref(function_ref)?;

        // -=-= Verification =-=-
        trace!(
            "Global function idx: {:?}",
            self.store.as_ref().unwrap_validated().modules[module_idx].functions[func_idx]
        );

        let func_inst_idx = *self.store.as_ref().unwrap_validated().modules[module_idx]
            .functions
            .get(func_idx)
            .ok_or(RuntimeError::FunctionNotFound)?;

        let func_inst = self
            .store
            .as_ref()
            .unwrap_validated()
            .functions
            .get(func_inst_idx)
            .ok_or(RuntimeError::FunctionNotFound)?;

        let func_ty = func_inst.ty();

        // Verify that the given parameters match the function parameters
        let param_types = params.iter().map(|v| v.to_ty()).collect::<Vec<_>>();

        if func_ty.params.valtypes != param_types {
            panic!("Invalid parameters for function");
        }

        // Verify that the given return types match the function return types
        if func_ty.returns.valtypes != ret_types {
            panic!("Invalid return types for function");
        }

        // Prepare a new stack with the locals for the entry function
        let mut stack = Stack::new();
        let locals = Locals::new(params.into_iter(), func_inst.locals.iter().cloned());
        stack.push_stackframe(module_idx, func_idx, &func_ty, locals, 0, 0);

        let mut currrent_module_idx = module_idx;

        // Run the interpreter
        run(
            &mut currrent_module_idx,
            &mut stack,
            EmptyHookSet,
            self.store.as_mut().unwrap_validated(),
        )?;

        let func_inst_idx = *self.store.as_ref().unwrap_validated().modules[module_idx]
            .functions
            .get(func_idx)
            .ok_or(RuntimeError::FunctionNotFound)?;

        let func_inst = self
            .store
            .as_ref()
            .unwrap_validated()
            .functions
            .get(func_inst_idx)
            .ok_or(RuntimeError::FunctionNotFound)?;

        let func_ty = func_inst.ty();

        // Pop return values from stack
        let return_values = func_ty
            .returns
            .valtypes
            .iter()
            .rev()
            .map(|ty| stack.pop_value(*ty))
            .collect::<Vec<Value>>();

        // Values are reversed because they were popped from stack one-by-one. Now reverse them back
        let reversed_values = return_values.into_iter().rev();
        let ret = reversed_values.collect();
        debug!("Successfully invoked function");
        Ok(ret)
    }

    /// Get the indicies of a module and function by their names.
    ///
    /// # Arguments
    /// - `module_name`: The module in which to find the function.
    /// - `function_name`: The name of the function to find inside the module. The function must be a local function and
    ///   not an import.
    ///
    /// # Returns
    /// - `Ok((module_idx, func_idx))`, where `module_idx` is the internal index of the module inside the
    ///   [RuntimeInstance], and `func_idx` is the internal index of the function inside the module.
    /// - `Err(RuntimeError::ModuleNotFound)`, if the module is not found.
    /// - `Err(RuntimeError::FunctionNotFound`, if the function is not found within the module.
    ///
    pub fn invoke_dynamic_unchecked_return_ty(
        &mut self,
        function_ref: &FunctionRef,
        params: Vec<Value>,
        store: &mut Store,
    ) -> Result<Vec<Value>, RuntimeError> {
        if self.store.is_none() {
            return Err(RuntimeError::StoreNotFound);
        }
        // First, verify that the function reference is valid
        let (module_idx, func_idx) = self.verify_function_ref(function_ref)?;

        // -=-= Verification =-=-
        let func_inst_idx = *self.store.as_ref().unwrap_validated().modules[module_idx]
            .functions
            .get(func_idx)
            .ok_or(RuntimeError::FunctionNotFound)?;

        let func_inst = self
            .store
            .as_ref()
            .unwrap_validated()
            .functions
            .get(func_inst_idx)
            .ok_or(RuntimeError::FunctionNotFound)?;

        let func_ty = func_inst.ty();

        // Verify that the given parameters match the function parameters
        let param_types = params.iter().map(|v| v.to_ty()).collect::<Vec<_>>();

        if func_ty.params.valtypes != param_types {
            panic!("Invalid parameters for function");
        }

        // Prepare a new stack with the locals for the entry function
        let mut stack = Stack::new();
        let locals = Locals::new(params.into_iter(), func_inst.locals.iter().cloned());
        stack.push_stackframe(module_idx, func_idx, &func_ty, locals, 0, 0);

        let mut currrent_module_idx = module_idx;

        // Run the interpreter
        run(
            &mut currrent_module_idx,
            &mut stack,
            EmptyHookSet,
            self.store.as_mut().unwrap_validated(),
        )?;

        let func_inst_idx = *self.store.as_ref().unwrap_validated().modules[module_idx]
            .functions
            .get(func_idx)
            .ok_or(RuntimeError::FunctionNotFound)?;

        let func_inst = store
            .functions
            .get(func_inst_idx)
            .ok_or(RuntimeError::FunctionNotFound)?;

        let func_ty = func_inst.ty();

        // Pop return values from stack
        let return_values = func_ty
            .returns
            .valtypes
            .iter()
            .rev()
            .map(|ty| stack.pop_value(*ty))
            .collect::<Vec<Value>>();

        // Values are reversed because they were popped from stack one-by-one. Now reverse them back
        let reversed_values = return_values.into_iter().rev();
        let ret = reversed_values.collect();
        debug!("Successfully invoked function");
        Ok(ret)
    }

    fn get_indicies(
        &self,
        module_name: &str,
        function_name: &str,
    ) -> Result<(usize, usize), RuntimeError> {
        if self.store.is_none() {
            return Err(RuntimeError::StoreNotFound);
        }

        let module_idx = self
            .store
            .as_ref()
            .unwrap_validated()
            .get_module_idx_from_name(module_name)?;

        let func_idx = self.store.as_ref().unwrap_validated().modules[module_idx]
            .exports
            .iter()
            .find_map(|export| {
                if export.name == function_name {
                    match export.desc {
                        ExportDesc::FuncIdx(func_idx) => Some(func_idx),
                        _ => None,
                    }
                } else {
                    None
                }
            })
            .ok_or(RuntimeError::FunctionNotFound)?;

        Ok((module_idx, func_idx))
    }

    /// Verify that the function reference is still valid. A function reference may be invalid if it created from
    /// another [RuntimeInstance] or the modules inside the instance have been changed in a way that the indicies inside
    /// the [FunctionRef] would be invalid.
    ///
    /// Note: this function ensures that making an unchecked indexation will not cause a panic.
    ///
    /// # Returns
    /// - `Ok((function_ref.module_idx, function_ref.func_idx))`
    /// - `Err(RuntimeError::FunctionNotFound)`, or `Err(RuntimeError::ModuleNotFound)` if the function is not valid.
    ///
    /// # Implementation details
    /// For an exported function (i.e. created by the same [RuntimeInstance]), the names are re-resolved using
    /// [RuntimeInstance::get_indicies], and the indicies are compared with the indicies in the [FunctionRef].
    ///
    /// For a [FunctionRef] with the [export](FunctionRef::exported) flag set to `false`, the indicies are checked to be
    /// in-bounds, and that the module name matches the module name in the [FunctionRef]. The function name is ignored.
    fn verify_function_ref(
        &self,
        function_ref: &FunctionRef,
    ) -> Result<(usize, usize), RuntimeError> {
        if self.store.is_none() {
            return Err(RuntimeError::StoreNotFound);
        }
        if function_ref.exported {
            let (module_idx, func_idx) =
                self.get_indicies(&function_ref.module_name, &function_ref.function_name)?;

            // TODO: figure out errors :)
            if module_idx != function_ref.module_index {
                return Err(RuntimeError::ModuleNotFound);
            }
            if func_idx != function_ref.function_index {
                return Err(RuntimeError::FunctionNotFound);
            }

            Ok((module_idx, func_idx))
        } else {
            let (module_idx, func_idx) = (function_ref.module_index, function_ref.function_index);

            let module = self
                .store
                .as_ref()
                .unwrap_validated()
                .modules
                .get(module_idx)
                .ok_or(RuntimeError::ModuleNotFound)?;

            if module.name != function_ref.module_name {
                return Err(RuntimeError::ModuleNotFound);
            }

            Ok((module_idx, func_idx))
        }
    }
}

/// Used for getting the offset of an address.
///
/// Related to the Active Elements
///
/// <https://webassembly.github.io/spec/core/syntax/modules.html#element-segments>
///
/// Since active elements need an offset given by a constant expression, in this case
/// they can only be an i32 (which can be understood from either a [`Value::I32`] - but
/// since we don't unbox the address of the reference, for us also a [`Value::Ref`] -
/// or from a Global)
fn get_address_offset(value: Value) -> Option<u32> {
    match value {
        Value::I32(val) => Some(val),
        Value::Ref(rref) => match rref {
            Ref::Extern(extern_addr) => extern_addr.addr.map(|addr| addr as u32),
            // TODO: fix
            Ref::Func(func_addr) => func_addr.addr.map(|addr| addr as u32),
        },
        // TODO: from wasmtime - implement only global
        _ => unreachable!(),
    }
}
