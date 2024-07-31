use alloc::vec::Vec;

use locals::Locals;
use value_stack::Stack;

use crate::core::indices::FuncIdx;
use crate::core::reader::types::export::Export;
use crate::core::reader::types::{FuncType, ValType};
use crate::core::reader::WasmReader;
use crate::execution::assert_validated::UnwrapValidatedExt;
use crate::execution::hooks::HookSet;
use crate::execution::store::{FuncInst, GlobalInst, MemInst, Store};
use crate::execution::value::Value;
use crate::validation::code::read_declared_locals;
use crate::value::InteropValueList;
use crate::RuntimeError;
use crate::ValidationInfo;

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

pub struct RuntimeInstance<'bytecode> {
    /// Reference to the bytecode
    pub wasm_bytecode: &'bytecode [u8],

    /// List of types from this WASM module
    types: Vec<FuncType>,

    /// Exported symbols
    exports: Vec<Export>,

    /// Store, i.e. for the Linear Memory
    store: Store,
}

impl<'bytecode> RuntimeInstance<'bytecode> {
    /// Create a new runtime instance based of a validation info
    ///
    /// This does not run the start function, hence the WASM might be not in a valid state for
    /// running arbitrary other functions.
    pub fn new(validation_info: &'_ ValidationInfo<'bytecode>) -> Result<Self, RuntimeError> {
        trace!("Starting instantiation of bytecode");

        let store = Self::init_store(validation_info);

        let mut instance = RuntimeInstance {
            wasm_bytecode: validation_info.wasm,
            types: validation_info.types.clone(),
            exports: validation_info.exports.clone(),
            store,
        };

        // TODO externalize this
        // if let Some(start) = validation_info.start {
        //     let result = instance.invoke_func::<(), ()>(start, ());
        //     result?;
        // }

        Ok(instance)
    }

    /// Initialize the store
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

    pub fn invoke_named<Param: InteropValueList, Returns: InteropValueList>(
        &mut self,
        func_name: &str,
        param: Param,
    ) -> Result<Returns, RuntimeError> {
        todo!();
        /*

        // TODO: Optimize this search for better than linear-time. Pre-processing will likely be required
        let func_idx = self.runtime_instance.exports.iter().find_map(|export| {
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
            Err(RuntimeError::FunctionNotFound.into())
        }
        */
    }

    /// Can only invoke functions with signature `[t1] -> [t2]` as of now.
    pub fn invoke_func<Param: InteropValueList, Returns: InteropValueList>(
        &mut self,
        func_idx: FuncIdx,
        params: Param,
    ) -> Result<Returns, RuntimeError> {
        todo!();
        /*
        // -=-= Verification =-=-
        let func_ty = self
            .runtime_instance
            .types
            .get(func_inst.ty)
            .unwrap_validated();

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
        */
    }

    /// Invokes a function with the given parameters, and return types which are not known at compile time.
    pub fn invoke_dynamic(
        &mut self,
        func_idx: FuncIdx,
        params: Vec<Value>,
        ret_types: &[ValType],
    ) -> Result<Vec<Value>, RuntimeError> {
        todo!();
        /*
        // -=-= Verification =-=-
        let func_inst = self
            .runtime_instance
            .store
            .funcs
            .get(func_idx)
            .expect("valid FuncIdx");
        let func_ty = self
            .runtime_instance
            .types
            .get(func_inst.ty)
            .unwrap_validated();

        // Verify that the given parameters match the function parameters
        let param_types = params.iter().map(|v| v.to_ty()).collect::<Vec<_>>();

        if func_ty.params.valtypes != param_types {
            panic!("Invalid parameters for function");
        }

        // Verify that the given return types match the function return types
        if func_ty.returns.valtypes != ret_types {
            panic!("Invalid return types for function");
        }

        let func_inst = self
            .runtime_instance
            .store
            .funcs
            .get(func_idx)
            .expect("valid FuncIdx");
        let func_ty = self
            .runtime_instance
            .types
            .get(func_inst.ty)
            .unwrap_validated()
            .clone();

        let return_values = self.function(func_idx, &params, &func_ty.returns.valtypes)?;

        debug!("Successfully invoked function");
        Ok(return_values)
        */
    }

    fn function(
        &mut self,
        idx: FuncIdx,
        params: &[Value],
        return_tys: &[ValType],
    ) -> Result<Vec<Value>, RuntimeError> {
        todo!();
        /*
        let mut stack = Stack::new();

        // Push parameters on stack
        for parameter in params {
            stack.push_value(parameter.clone());
        }

        self.push_callframe(idx, &mut stack)?;
        self.run(idx, &mut stack)?;

        let mut return_values = return_tys
            .iter()
            .map(|ty| stack.pop_value(*ty))
            .collect::<Vec<Value>>();

        // These end up in wrong order (top element on the stack becomes idx 0)
        return_values.reverse();
        // Pop return values from stack
        Ok(return_values)
        */
    }
}

pub struct Runner<'a, 'bytecode, H>
where
    H: HookSet,
{
    /// Handle to the mostly immutable (except for the store) RutimeInstance
    pub runtime_instance: &'a mut RuntimeInstance<'bytecode>,

    /// Program counter/Instrunction pointer to the wasm bytecode
    pub wasm_reader: WasmReader<'bytecode>,

    ///  Optional Hooks to be called whenever *something* happens
    pub hook_set: H,

    /// State of this runner
    pub(crate) state: RunnerState<'bytecode>,

    /// Remaining fuel
    pub fuel: u128,

    /// Parameter-/Return-values for before invoking/after completion of a function
    ///
    /// Not to be used during actual interpretation!
    pub value_stack: Vec<Value>,

    pub stack: Stack,
}

/// Possible states of this
pub(crate) enum RunnerState<'a> {
    Uninitialized,
    ReadyToRun {
        func_idx: FuncIdx,
    },
    InProgress {
        call_stack: Vec<CallFrame<'a>>,
        func_idx: FuncIdx,
    },
    ReturnValueAvailable {
        func_idx: FuncIdx,
    },
}

impl<'a> RunnerState<'a> {}

pub enum RunResult {
    Uninitialized,
    InProgress,
    Done,
}

/// OLD

impl<'a, 'bytecode, H> Runner<'a, 'bytecode, H>
where
    H: HookSet,
{
    /// Set the WASM function to run and check that the types on the value stack match
    pub fn set_wasm_fn(&mut self, func_idx: FuncIdx) -> Result<(), RuntimeError> {
        // TODO should this function be able to abort the current execution?

        // get the function from the store
        let func = self
            .runtime_instance
            .store
            .funcs
            .get(func_idx)
            .ok_or(RuntimeError::FunctionNotFound)?;

        // retrieve the function's signature
        let func_param_ty = &self
            .runtime_instance
            .types
            .get(func.ty)
            .expect("function type not in store, but validation guarantees this never happens")
            .params
            .valtypes;

        // check that the value stack has enough values, having more than required is fine as long as the types match
        if func_param_ty.len() > self.value_stack.len() {
            error!(
                "to few params on valuestack to call function {func_idx}: {} needed, got only {}",
                func_param_ty.len(),
                self.value_stack.len()
            );
            return Err(RuntimeError::TypeMismatch);
        }

        // split of that part of the value_stack's tail that makes up the function parameters
        let (_, param_values) = self
            .value_stack
            .split_at(self.value_stack.len() - func_param_ty.len());

        assert_eq!(param_values.len(), func_param_ty.len());

        // check that the tail of the value_stack matches the type signature of the function parameters
        for (i, expected_param_type) in func_param_ty.iter().enumerate() {
            let stack_value_ty = param_values[i].to_ty();
            if *expected_param_type != stack_value_ty {
                error!("parameter {i} to function {func_idx} has wrong type, expected {expected_param_type:?} but got {stack_value_ty:?}");
                return Err(RuntimeError::TypeMismatch);
            }
        }

        // store that we are now ready to run the function
        self.state = RunnerState::ReadyToRun { func_idx };

        Ok(())
    }
}

/// Push a new [CallFrame] to the call-frame stack
fn push_callframe<'bytecode>(
    call_stack: &mut Vec<CallFrame<'bytecode>>,
    runtime_instance: &RuntimeInstance<'bytecode>,
    idx: FuncIdx,
    stack: &mut Stack,
) -> Result<(), RuntimeError> {
    let inst = runtime_instance.store.funcs.get(idx).unwrap_validated();

    // Pop parameters from stack
    let func_type = runtime_instance.types.get(inst.ty).unwrap_validated();
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
    let mut wasm = WasmReader::new(runtime_instance.wasm_bytecode);
    wasm.move_start_to(inst.code_expr)
        .expect("unable to move pc to function start, but validation guarantees this works");

    let call_frame = CallFrame {
        locals,
        function_idx: idx,
        reader: wasm,
    };
    call_stack.push(call_frame);

    Ok(())
}

/// Pop a call frame, e.g. when returning from a function
///
/// Returns true if there is at least one remaining [CallFrame]
fn pop_callframe(call_stack: &mut Vec<CallFrame>) {
    // TODO maybe when we return from inside nested control blocks there is more cleanup todo?
    assert!(
        call_stack.pop().is_some(),
        "popping the CallStack when it was empty is a logic error"
    );
}
