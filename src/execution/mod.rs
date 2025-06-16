use core::marker::PhantomData;

use alloc::vec::Vec;

use const_interpreter_loop::run_const_span;
use function_ref::FunctionRef;
use interpreter_loop::run;
use locals::Locals;
use value_stack::Stack;

use crate::execution::assert_validated::UnwrapValidatedExt;
use crate::execution::hooks::{EmptyHookSet, HookSet};
use crate::execution::store::Store;
use crate::execution::value::Value;
use crate::value::InteropValueList;
use crate::{Result as CustomResult, RuntimeError, ValType, ValidationInfo};

pub(crate) mod assert_validated;
pub mod const_interpreter_loop;
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

pub enum InvocationState<'b, Returns: InteropValueList, H: HookSet> {
    Finished(Returns, RuntimeInstance<'b, H, Finished>),
    OutOfFuel(RuntimeInstance<'b, H, Resumable>),
    Canceled(RuntimeInstance<'b, H, Canceled>),
}

impl<'b, Returns: InteropValueList, H: HookSet> InvocationState<'b, Returns, H> {
    pub fn is_finished(self) -> Option<Returns> {
        match self {
            InvocationState::Finished(returns, _) => Some(returns),
            _ => None,
        }
    }
}

trait RuntimeState {
    type STATE;
}

pub struct Invocable;
impl RuntimeState for Invocable {
    type STATE = ();
}

pub struct Finished;
impl RuntimeState for Finished {
    type STATE = FinishedState;
}

pub struct Resumable;
impl RuntimeState for Resumable {
    type STATE = ResumableState;
}

pub struct Canceled;
impl RuntimeState for Canceled {
    type STATE = CanceledState;
}

struct FinishedState {
    remaining_fuel: Option<usize>,
}

struct ResumableState {
    stack: Stack,
    current_module_idx: usize,
    current_fuel: Option<usize>,
}

struct CanceledState {
    remaining_fuel: Option<usize>,
}

pub struct RuntimeInstance<'b, H = EmptyHookSet, S = Invocable>
where
    H: HookSet,
    S: RuntimeState,
{
    pub hook_set: H,
    pub store: Option<Store<'b>>,

    state: S::STATE,

    pub _phantom: PhantomData<S>,
}

impl<'b> RuntimeInstance<'b, EmptyHookSet, Invocable> {
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

impl<'b, H> RuntimeInstance<'b, H, Invocable>
where
    H: HookSet,
{
    pub fn add_module(
        &mut self,
        module_name: &str,
        validation_info: &'_ ValidationInfo<'b>,
    ) -> CustomResult<()> {
        match self.store {
            // TODO fix error
            None => return Err(crate::Error::RuntimeError(RuntimeError::ModuleNotFound)),
            Some(ref mut store) => {
                store.add_module(module_name, validation_info)?;
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

        let mut instance = RuntimeInstance {
            hook_set,
            store,
            state: (),
            _phantom: PhantomData,
        };
        instance.add_module(module_name, validation_info)?;

        Ok(instance)
    }

    pub fn get_function_by_name(
        &self,
        module_name: &str,
        function_name: &str,
    ) -> Result<FunctionRef, RuntimeError> {
        // TODO fix error
        let store = self.store.as_ref().ok_or(RuntimeError::ModuleNotFound)?;
        if !store.module_names.contains_key(module_name) {
            return Err(RuntimeError::ModuleNotFound);
        };
        FunctionRef::new_from_name(module_name, function_name, store)
            .map_err(|_| RuntimeError::FunctionNotFound)
    }

    pub fn get_function_by_index(
        &self,
        module_addr: usize,
        function_idx: usize,
    ) -> Result<FunctionRef, RuntimeError> {
        // TODO fix error
        let store = self.store.as_ref().ok_or(RuntimeError::ModuleNotFound)?;

        let module_inst = store
            .modules
            .get(module_addr)
            .ok_or(RuntimeError::ModuleNotFound)?;
        let func_addr = *module_inst
            .func_addrs
            .get(function_idx)
            .ok_or(RuntimeError::FunctionNotFound)?;

        Ok(FunctionRef { func_addr })
    }

    pub fn invoke<Param: InteropValueList, Returns: InteropValueList>(
        self,
        function_ref: &FunctionRef,
        params: Param,
        fuel: Option<usize>,
        // store: &mut Store,
    ) -> Result<InvocationState<'b, Returns, H>, RuntimeError> {
        // TODO fix error
        let store = self.store.as_ref().ok_or(RuntimeError::ModuleNotFound)?;

        let FunctionRef { func_addr } = *function_ref;
        let func_inst = store
            .functions
            .get(func_addr)
            .ok_or(RuntimeError::FunctionNotFound)?;

        let module_addr = func_inst.module_addr;

        // TODO handle this bad linear search that is unavoidable
        let (func_idx, _) = store.modules[module_addr]
            .func_addrs
            .iter()
            .enumerate()
            .find(|&(_idx, addr)| *addr == func_addr)
            .ok_or(RuntimeError::FunctionNotFound)?;

        let func_ty = func_inst.ty();

        // Check correct function parameters and return types
        if func_ty.params.valtypes != Param::TYS {
            panic!("Invalid `Param` generics");
        }
        if func_ty.returns.valtypes != Returns::TYS {
            panic!(
                "Invalid `Returns` generics, expected {:?}",
                func_ty.returns.valtypes
            );
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
            module_addr,
            func_idx,
            &func_ty,
            locals,
            usize::MAX,
            usize::MAX,
        );

        let current_module_idx = module_addr;

        let resumable = RuntimeInstance::<'b, H, Resumable> {
            hook_set: self.hook_set,
            store: self.store,
            state: ResumableState {
                stack,
                current_module_idx,
                current_fuel: fuel,
            },
            _phantom: PhantomData,
        };

        resumable.resume::<Returns>()
    }

    /// Invokes a function with the given parameters, and return types which are not known at compile time.
    pub fn invoke_dynamic(
        &mut self,
        function_ref: &FunctionRef,
        params: Vec<Value>,
        ret_types: &[ValType],
        // store: &mut Store,
    ) -> Result<Vec<Value>, RuntimeError> {
        // TODO fix error
        let store = self.store.as_ref().ok_or(RuntimeError::ModuleNotFound)?;

        let FunctionRef { func_addr } = *function_ref;
        let func_inst = store
            .functions
            .get(func_addr)
            .ok_or(RuntimeError::FunctionNotFound)?;

        let module_addr = func_inst.module_addr;

        // TODO handle this bad linear search that is unavoidable
        let (func_idx, _) = store.modules[module_addr]
            .func_addrs
            .iter()
            .enumerate()
            .find(|&(_idx, addr)| *addr == func_addr)
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
        stack.push_stackframe(module_addr, func_idx, &func_ty, locals, 0, 0);

        let mut currrent_module_idx = module_addr;

        // Run the interpreter
        run(
            &mut currrent_module_idx,
            &mut stack,
            EmptyHookSet,
            self.store.as_mut().unwrap_validated(),
            &mut None,
        )?;

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
    ) -> Result<Vec<Value>, RuntimeError> {
        // TODO fix error
        let store = self.store.as_ref().ok_or(RuntimeError::ModuleNotFound)?;

        let FunctionRef { func_addr } = *function_ref;
        let func_inst = store
            .functions
            .get(func_addr)
            .ok_or(RuntimeError::FunctionNotFound)?;

        let module_addr = func_inst.module_addr;

        // TODO handle this bad linear search that is unavoidable
        let (func_idx, _) = store.modules[module_addr]
            .func_addrs
            .iter()
            .enumerate()
            .find(|&(_idx, addr)| *addr == func_addr)
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
        stack.push_stackframe(module_addr, func_idx, &func_ty, locals, 0, 0);

        let mut currrent_module_idx = module_addr;

        // Run the interpreter
        run(
            &mut currrent_module_idx,
            &mut stack,
            EmptyHookSet,
            self.store.as_mut().unwrap_validated(),
            &mut None,
        )?;

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
}

impl<'b, H: HookSet> RuntimeInstance<'b, H, Resumable> {
    pub fn resume<Returns: InteropValueList>(
        mut self,
    ) -> Result<InvocationState<'b, Returns, H>, RuntimeError> {
        // Run the interpreter
        let state = run(
            &mut self.state.current_module_idx,
            &mut self.state.stack,
            H::default(),
            self.store.as_mut().unwrap_validated(),
            &mut self.state.current_fuel,
        )?;

        let state = match state {
            interpreter_loop::RunState::OutOfFuel => {
                let inst = RuntimeInstance::<'b, H, Resumable> {
                    hook_set: self.hook_set,
                    store: self.store,
                    state: self.state,
                    _phantom: PhantomData,
                };

                InvocationState::OutOfFuel(inst)
            }
            interpreter_loop::RunState::Finished => {
                let vals = Returns::TYS
                    .iter()
                    .rev()
                    .map(|ty| self.state.stack.pop_value(*ty))
                    .rev();

                let return_values = Returns::from_values(vals);

                let inst = RuntimeInstance::<'b, H, Finished> {
                    hook_set: self.hook_set,
                    store: self.store,
                    state: FinishedState {
                        remaining_fuel: self.state.current_fuel,
                    },
                    _phantom: PhantomData,
                };

                debug!("Successfully invoked function");
                InvocationState::Finished(return_values, inst)
            }
        };

        Ok(state)
    }

    pub fn cancel(self) -> InvocationState<'b, (), H> {
        InvocationState::Canceled(RuntimeInstance::<H, Canceled> {
            hook_set: self.hook_set,
            store: self.store,
            state: CanceledState {
                remaining_fuel: self.state.current_fuel,
            },
            _phantom: PhantomData,
        })
    }

    pub fn get_fuel(&self) -> Option<usize> {
        self.state.current_fuel
    }

    pub fn set_fuel(&mut self, fuel: Option<usize>) {
        self.state.current_fuel = fuel
    }
}

impl<'b, H: HookSet> RuntimeInstance<'b, H, Finished> {
    pub fn get_fuel(&self) -> Option<usize> {
        self.state.remaining_fuel
    }

    pub fn reset(self) -> RuntimeInstance<'b, H, Invocable> {
        return RuntimeInstance::<'b, H, Invocable> {
            hook_set: self.hook_set,
            store: self.store,
            state: (),
            _phantom: PhantomData,
        };
    }
}

impl<'b, H: HookSet> RuntimeInstance<'b, H, Canceled> {
    pub fn get_fuel(&self) -> Option<usize> {
        self.state.remaining_fuel
    }

    pub fn reset(self) -> RuntimeInstance<'b, H, Invocable> {
        return RuntimeInstance::<'b, H, Invocable> {
            hook_set: self.hook_set,
            store: self.store,
            state: (),
            _phantom: PhantomData,
        };
    }
}
