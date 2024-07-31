use alloc::vec::Vec;

use interpreter_loop::run;
use value_stack::Stack;

use crate::core::indices::FuncIdx;
use crate::core::reader::types::export::{Export, ExportDesc};
use crate::core::reader::types::{FuncType, ValType};
use crate::core::reader::WasmReader;
use crate::execution::assert_validated::UnwrapValidatedExt;
use crate::execution::hooks::{EmptyHookSet, HookSet};
use crate::execution::store::{FuncInst, GlobalInst, MemInst, Store};
use crate::execution::value::Value;
use crate::validation::code::read_declared_locals;
use crate::value::InteropValueList;
use crate::Error::RuntimeError;
use crate::RuntimeError::FunctionNotFound;
use crate::{Result, ValidationInfo};

// TODO
pub(crate) mod assert_validated;
pub mod hooks;
mod interpreter_loop;
pub(crate) mod label;
pub(crate) mod locals;
pub(crate) mod store;
pub mod value;
pub mod value_stack;

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
    pub fn new(validation_info: &'_ ValidationInfo<'b>) -> Result<Self> {
        Self::new_with_hooks(validation_info, EmptyHookSet)
    }
}

impl<'b, H> RuntimeInstance<'b, H>
where
    H: HookSet,
{
    pub fn new_with_hooks(validation_info: &'_ ValidationInfo<'b>, hook_set: H) -> Result<Self> {
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
            let result = instance.invoke_func::<(), ()>(start, ());
            result?;
        }

        Ok(instance)
    }

    pub fn invoke_named<Param: InteropValueList, Returns: InteropValueList>(
        &mut self,
        func_name: &str,
        param: Param,
    ) -> Result<Returns> {
        // TODO: Optimize this search for better than linear-time. Pre-processing will likely be required
        let func_idx = self.exports.iter().find_map(|export| {
            if export.name == func_name {
                match export.desc {
                    ExportDesc::FuncIdx(idx) => Some(idx),
                    _ => None,
                }
            } else {
                None
            }
        });

        if let Some(func_idx) = func_idx {
            self.invoke_func(func_idx, param)
        } else {
            Err(RuntimeError(FunctionNotFound))
        }
    }

    /// Can only invoke functions with signature `[t1] -> [t2]` as of now.
    pub fn invoke_func<Param: InteropValueList, Returns: InteropValueList>(
        &mut self,
        func_idx: FuncIdx,
        params: Param,
    ) -> Result<Returns> {
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

        // -=-= Invoke the function =-=-
        let mut stack = Stack::new();

        // Push parameters on stack
        for parameter in params.into_values() {
            stack.push_value(parameter);
        }

        run(
            &self.types,
            self.wasm_bytecode,
            &mut self.store,
            func_idx,
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
        func_idx: FuncIdx,
        params: Vec<Value>,
        ret_types: &[ValType],
    ) -> Result<Vec<Value>> {
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

        // -=-= Invoke the function =-=-
        let mut stack = Stack::new();

        // Push parameters on stack
        for parameter in params {
            stack.push_value(parameter);
        }

        run(
            &self.types,
            self.wasm_bytecode,
            &mut self.store,
            func_idx,
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

    fn init_store(validation_info: &ValidationInfo) -> Store {
        let function_instances: Vec<FuncInst> = {
            let mut wasm_reader = WasmReader::new(validation_info.wasm);

            let functions = validation_info.functions.iter();
            let func_blocks = validation_info.func_blocks.iter();

            functions
                .zip(func_blocks)
                .map(|(ty, func)| {
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
            .map(|global| {
                // TODO execute `global.init_expr` to get initial value. For now just use a default value.
                let value = Value::default_from_ty(global.ty.ty);

                GlobalInst {
                    global: *global,
                    value,
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
