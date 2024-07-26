use alloc::vec::Vec;

use locals::Locals;
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
use crate::RuntimeError::FunctionNotFound;
use crate::{Result, ValidationInfo};

// TODO
pub(crate) mod assert_validated;
pub mod hooks;
mod instructions;
pub(crate) mod label;
pub(crate) mod locals;
pub(crate) mod store;
pub mod value;
pub mod value_stack;

struct CallFrame<'a> {
    locals: Locals,
    #[allow(dead_code)]
    function_idx: FuncIdx,
    reader: WasmReader<'a>,
}

pub struct RuntimeInstance<'b, H = EmptyHookSet>
where
    H: HookSet,
{
    pub wasm_bytecode: &'b [u8],
    types: Vec<FuncType>,
    exports: Vec<Export>,
    store: Store,
    call_stack: Vec<CallFrame<'b>>,
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
            call_stack: Vec::new(),
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
            Err(FunctionNotFound.into())
        }
    }

    /// Can only invoke functions with signature `[t1] -> [t2]` as of now.
    pub fn invoke_func<Param: InteropValueList, Returns: InteropValueList>(
        &mut self,
        func_idx: FuncIdx,
        params: Param,
    ) -> Result<Returns> {
        // -=-= Verification =-=-
        let func_inst = self.store.funcs.get(func_idx).ok_or(FunctionNotFound)?;
        let func_ty = self.types.get(func_inst.ty).unwrap_validated();

        // Check correct function parameters and return types
        if func_ty.params.valtypes != Param::TYS {
            panic!("Invalid `Param` generics");
        }
        if func_ty.returns.valtypes != Returns::TYS {
            panic!("Invalid `Returns` generics");
        }

        // Invoke the function
        let return_values = self.function(func_idx, &params.into_values(), Returns::TYS)?;
        debug!("Successfully invoked function");
        Ok(Returns::from_values(return_values.into_iter()))
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

        let func_inst = self.store.funcs.get(func_idx).expect("valid FuncIdx");
        let func_ty = self.types.get(func_inst.ty).unwrap_validated().clone();

        let return_values = self.function(func_idx, &params, &func_ty.returns.valtypes)?;

        debug!("Successfully invoked function");
        Ok(return_values)
    }

    fn function(
        &mut self,
        idx: FuncIdx,
        params: &[Value],
        return_tys: &[ValType],
    ) -> Result<Vec<Value>> {
        let mut stack = Stack::new();

        // Push parameters on stack
        for parameter in params {
            stack.push_value(parameter.clone());
        }

        self.push_callframe(idx, &mut stack)?;
        self.run(&mut stack)?;

        let mut return_values = return_tys
            .iter()
            .map(|ty| stack.pop_value(*ty))
            .collect::<Vec<Value>>();

        // These end up in wrong order (top element on the stack becomes idx 0)
        return_values.reverse();
        // Pop return values from stack
        Ok(return_values)
    }

    /// Push a new [CallFrame] to the call-frame stack
    fn push_callframe(&mut self, idx: FuncIdx, stack: &mut Stack) -> Result<()> {
        let inst = self.store.funcs.get(idx).unwrap_validated();

        // Pop parameters from stack
        let func_type = self.types.get(inst.ty).unwrap_validated();
        let mut params: Vec<Value> = func_type
            .params
            .valtypes
            .iter()
            .map(|ty| stack.pop_value(*ty))
            .collect();
        params.reverse();

        // Create locals from parameters and declared locals
        let locals = Locals::new(params.into_iter(), inst.locals.iter().cloned());

        // Start reading the function's instructions
        let mut wasm = WasmReader::new(self.wasm_bytecode);
        wasm.move_start_to(inst.code_expr)?;

        let call_frame = CallFrame {
            locals,
            function_idx: idx,
            reader: wasm,
        };
        self.call_stack.push(call_frame);

        Ok(())
    }

    /// Pop a call frame, e.g. when returning from a function
    ///
    /// Returns true if there is at least one remaining [CallFrame]
    fn pop_callframe(&mut self) {
        // TODO maybe when we return from inside nested control blocks there is more cleanup todo?
        assert!(
            self.call_stack.pop().is_some(),
            "popping the CallStack when it was empty is a logic error"
        );
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
