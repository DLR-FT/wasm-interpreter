use alloc::string::ToString;
use alloc::vec::Vec;

use const_interpreter_loop::run_const;
use function_ref::FunctionRef;
use interpreter_loop::run;
use locals::Locals;
use value_stack::Stack;

use crate::core::reader::types::export::{Export, ExportDesc};
use crate::core::reader::types::FuncType;
use crate::core::reader::WasmReader;
use crate::execution::assert_validated::UnwrapValidatedExt;
use crate::execution::hooks::{EmptyHookSet, HookSet};
use crate::execution::store::{FuncInst, GlobalInst, MemInst, Store};
use crate::execution::value::Value;
use crate::validation::code::read_declared_locals;
use crate::value::InteropValueList;
use crate::{RuntimeError, ValType, ValidationInfo};

// TODO
pub(crate) mod assert_validated;
mod const_interpreter_loop;
pub mod function_ref;
pub mod hooks;
mod interpreter_loop;
pub(crate) mod locals;
pub(crate) mod store;
pub mod value;
pub mod value_stack;

/// The default module name if a [RuntimeInstance] was created using [RuntimeInstance::new].
pub const DEFAULT_MODULE: &str = "__interpreter_default__";

pub struct RuntimeInstance<'b, H = EmptyHookSet>
where
    H: HookSet,
{
    pub wasm_bytecode: &'b [u8],
    types: Vec<FuncType>,
    exports: Vec<Export>,
    store: Store,
    pub hook_set: H,
}

impl<'b> RuntimeInstance<'b, EmptyHookSet> {
    pub fn new(validation_info: &'_ ValidationInfo<'b>) -> Result<Self, RuntimeError> {
        Self::new_with_hooks(DEFAULT_MODULE, validation_info, EmptyHookSet)
    }

    pub fn new_named(
        module_name: &str,
        validation_info: &'_ ValidationInfo<'b>,
    ) -> Result<Self, RuntimeError> {
        Self::new_with_hooks(module_name, validation_info, EmptyHookSet)
    }
}

impl<'b, H> RuntimeInstance<'b, H>
where
    H: HookSet,
{
    pub fn new_with_hooks(
        module_name: &str,
        validation_info: &'_ ValidationInfo<'b>,
        hook_set: H,
    ) -> Result<Self, RuntimeError> {
        trace!("Starting instantiation of bytecode");

        let store = Self::init_store(validation_info);

        let mut instance = RuntimeInstance {
            wasm_bytecode: validation_info.wasm,
            types: validation_info.types.clone(),
            exports: validation_info.exports.clone(),
            store,
            hook_set,
        };

        if let Some(start) = validation_info.start {
            // "start" is not always exported, so we need create a non-API exposed function reference.
            // Note: function name is not important here, as it is not used in the verification process.
            let start_fn = FunctionRef {
                module_name: module_name.to_string(),
                function_name: "start".to_string(),
                module_index: 0,
                function_index: start,
                exported: false,
            };
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
        // TODO: Module resolution
        let function_name = self
            .exports
            .iter()
            .find(|export| match &export.desc {
                ExportDesc::FuncIdx(idx) => *idx == function_idx,
                _ => false,
            })
            .map(|export| export.name.clone())
            .ok_or(RuntimeError::FunctionNotFound)?;

        Ok(FunctionRef {
            // TODO: get the module name from the module index
            module_name: DEFAULT_MODULE.to_string(),
            function_name,
            module_index: module_idx,
            function_index: function_idx,
            exported: true,
        })
    }

    // TODO: remove this annotation when implementing the function
    #[allow(clippy::result_unit_err)]
    pub fn add_module(
        &mut self,
        _module_name: &str,
        _validation_info: &'_ ValidationInfo<'b>,
    ) -> Result<(), ()> {
        todo!("Implement module linking");
    }

    pub fn invoke<Param: InteropValueList, Returns: InteropValueList>(
        &mut self,
        function_ref: &FunctionRef,
        params: Param,
    ) -> Result<Returns, RuntimeError> {
        // First, verify that the function reference is valid
        let (_module_idx, func_idx) = self.verify_function_ref(function_ref)?;

        // -=-= Verification =-=-
        let func_inst = self.store.funcs.get(func_idx).expect("valid FuncIdx");
        let func_ty = self.types.get(func_inst.ty).unwrap_validated();

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
        stack.push_stackframe(func_idx, func_ty, locals, usize::MAX, func_inst.sidetable.clone(), 0);

        // Run the interpreter
        run(
            self.wasm_bytecode,
            &self.types,
            &mut self.store,
            &mut stack,
            EmptyHookSet,
        )?;

        // Pop return values from stack
        let return_values = Returns::TYS
            .iter()
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
    ) -> Result<Vec<Value>, RuntimeError> {
        // First, verify that the function reference is valid
        let (_module_idx, func_idx) = self.verify_function_ref(function_ref)?;

        // -=-= Verification =-=-
        let func_inst = self.store.funcs.get(func_idx).expect("valid FuncIdx");
        let func_ty = self.types.get(func_inst.ty).unwrap_validated();

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
        stack.push_stackframe(func_idx, func_ty, locals, 0, func_inst.sidetable.clone(), 0);

        // Run the interpreter
        run(
            self.wasm_bytecode,
            &self.types,
            &mut self.store,
            &mut stack,
            EmptyHookSet,
        )?;

        let func_inst = self.store.funcs.get(func_idx).expect("valid FuncIdx");
        let func_ty = self.types.get(func_inst.ty).unwrap_validated();

        // Pop return values from stack
        let return_values = func_ty
            .returns
            .valtypes
            .iter()
            .map(|ty| stack.pop_value(*ty))
            .collect::<Vec<Value>>();

        // Values are reversed because they were popped from stack one-by-one. Now reverse them back
        let reversed_values = return_values.into_iter().rev();
        let ret = reversed_values.collect();
        debug!("Successfully invoked function");
        Ok(ret)
    }

    // TODO: replace this with the lookup table when implmenting the linker
    fn get_indicies(
        &self,
        _module_name: &str,
        function_name: &str,
    ) -> Result<(usize, usize), RuntimeError> {
        let func_idx = self
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

        Ok((0, func_idx))
    }

    fn verify_function_ref(
        &self,
        function_ref: &FunctionRef,
    ) -> Result<(usize, usize), RuntimeError> {
        if function_ref.exported {
            let (module_idx, func_idx) =
                self.get_indicies(&function_ref.module_name, &function_ref.function_name)?;

            if module_idx != function_ref.module_index || func_idx != function_ref.function_index {
                // TODO: should we return a different error here?
                return Err(RuntimeError::FunctionNotFound);
            }

            Ok((module_idx, func_idx))
        } else {
            let (module_idx, func_idx) = (function_ref.module_index, function_ref.function_index);

            // TODO: verify module named - index mapping.

            // Sanity check that the function index is at least in the bounds of the store, though this doesn't mean
            // that it's a valid function.
            self.store
                .funcs
                .get(func_idx)
                .ok_or(RuntimeError::FunctionNotFound)?;

            Ok((module_idx, func_idx))
        }
    }

    fn init_store(validation_info: &ValidationInfo) -> Store {
        let function_instances: Vec<FuncInst> = {
            let mut wasm_reader = WasmReader::new(validation_info.wasm);

            let functions = validation_info.functions.iter();
            let func_blocks = validation_info.func_blocks.iter();

            functions
                .zip(func_blocks)
                .map(|(ty, (func, sidetable))| {
                    wasm_reader
                        .move_start_to(*func)
                        .expect("function index to be in the bounds of the WASM binary");

                    let (locals, bytes_read) = wasm_reader
                        .measure_num_read_bytes(read_declared_locals)
                        .unwrap_validated();

                    let code_expr = wasm_reader
                        .make_span(func.len() - bytes_read)
                        .expect("TODO remove this expect");

                    FuncInst {
                        ty: *ty,
                        locals,
                        code_expr,
                        sidetable: sidetable.clone(),
                    }
                })
                .collect()
        };

        let memory_instances: Vec<MemInst> = validation_info
            .memories
            .iter()
            .map(|ty| MemInst::new(*ty))
            .collect();

        let global_instances: Vec<GlobalInst> = validation_info
            .globals
            .iter()
            .map({
                let mut stack = Stack::new();
                move |global| {
                    let mut wasm = WasmReader::new(validation_info.wasm);
                    // The place we are moving the start to should, by all means, be inside the wasm bytecode.
                    wasm.move_start_to(global.init_expr).unwrap_validated();
                    // We shouldn't need to clear the stack. If validation is correct, it will remain empty after execution.

                    run_const(wasm, &mut stack, ());
                    let value = stack.pop_value(global.ty.ty);

                    GlobalInst {
                        global: *global,
                        value,
                    }
                }
            })
            .collect();

        Store {
            funcs: function_instances,
            mems: memory_instances,
            globals: global_instances,
        }
    }
}
