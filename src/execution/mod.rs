use crate::core::reader::types::global::GlobalType;
use crate::resumable::{ResumableRef, RunState};
use alloc::borrow::ToOwned;
use alloc::vec::Vec;

use const_interpreter_loop::run_const_span;
use function_ref::FunctionRef;
use value_stack::Stack;

use crate::core::reader::types::{FuncType, ResultType};
use crate::execution::assert_validated::UnwrapValidatedExt;
use crate::execution::config::Config;
use crate::execution::store::Store;
use crate::execution::value::Value;
use crate::interop::InteropValueList;
use crate::{RuntimeError, ValidationInfo};

pub(crate) mod assert_validated;
pub mod config;
pub mod const_interpreter_loop;
pub mod error;
pub mod function_ref;
pub mod interop;
mod interpreter_loop;
pub(crate) mod linear_memory;
pub(crate) mod little_endian;
pub mod registry;
pub mod resumable;
pub mod store;
pub mod value;
pub mod value_stack;

/// The default module name if a [RuntimeInstance] was created using [RuntimeInstance::new].
pub const DEFAULT_MODULE: &str = "__interpreter_default__";

pub struct RuntimeInstance<'b, T: Config = ()> {
    pub store: Store<'b, T>,
}

impl<T: Config + Default> Default for RuntimeInstance<'_, T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<'b, T: Config> RuntimeInstance<'b, T> {
    pub fn new(user_data: T) -> Self {
        RuntimeInstance {
            store: Store::new(user_data),
        }
    }

    pub fn new_with_default_module(
        user_data: T,
        validation_info: &'_ ValidationInfo<'b>,
    ) -> Result<Self, RuntimeError> {
        let mut instance = Self::new(user_data);
        instance.add_module(DEFAULT_MODULE, validation_info)?;
        Ok(instance)
    }

    pub fn new_named(
        user_data: T,
        module_name: &str,
        validation_info: &'_ ValidationInfo<'b>,
        // store: &mut Store,
    ) -> Result<Self, RuntimeError> {
        let mut instance = Self::new(user_data);
        instance.add_module(module_name, validation_info)?;
        Ok(instance)
    }

    pub fn add_module(
        &mut self,
        module_name: &str,
        validation_info: &'_ ValidationInfo<'b>,
    ) -> Result<(), RuntimeError> {
        self.store.add_module(module_name, validation_info, None)
    }

    pub fn get_function_by_name(
        &self,
        module_name: &str,
        function_name: &str,
    ) -> Result<FunctionRef, RuntimeError> {
        FunctionRef::new_from_name(module_name, function_name, &self.store)
            .map_err(|_| RuntimeError::FunctionNotFound)
    }

    pub fn get_function_by_index(
        &self,
        module_addr: usize,
        function_idx: usize,
    ) -> Result<FunctionRef, RuntimeError> {
        let module_inst = self
            .store
            .modules
            .get(module_addr)
            .ok_or(RuntimeError::ModuleNotFound)?;
        let func_addr = *module_inst
            .func_addrs
            .get(function_idx)
            .ok_or(RuntimeError::FunctionNotFound)?;

        Ok(FunctionRef { func_addr })
    }

    /// Invokes a function with the given parameters of type `Param`, and return types of type `Returns`.
    pub fn invoke_typed<Params: InteropValueList, Returns: InteropValueList>(
        &mut self,
        function_ref: &FunctionRef,
        params: Params,
        // store: &mut Store,
    ) -> Result<Returns, RuntimeError> {
        self.invoke(function_ref, params.into_values())
            .map(|values| Returns::try_from_values(values.into_iter()).unwrap_validated())
    }

    /// Invokes a function with the given parameters. The return types depend on the function signature.
    pub fn invoke(
        &mut self,
        function_ref: &FunctionRef,
        params: Vec<Value>,
    ) -> Result<Vec<Value>, RuntimeError> {
        let FunctionRef { func_addr } = *function_ref;
        self.store
            .invoke(func_addr, params, None)
            .map(|run_state| match run_state {
                RunState::Finished(values) => values,
                _ => unreachable!("non metered invoke call"),
            })
    }

    /// Creates a new resumable, which when resumed for the first time invokes the function `function_ref` is associated
    /// to, with the arguments `params`. The newly created resumable initially stores `fuel` units of fuel. Returns a
    /// `[ResumableRef]` associated to the newly created resumable on success.
    pub fn create_resumable(
        &self,
        function_ref: &FunctionRef,
        params: Vec<Value>,
        fuel: u32,
    ) -> Result<ResumableRef, RuntimeError> {
        let FunctionRef { func_addr } = *function_ref;
        self.store.create_resumable(func_addr, params, Some(fuel))
    }

    /// resumes the resumable associated to `resumable_ref`. Returns a `[RunState]` associated to this resumable  if the
    /// resumable ran out of fuel or completely executed.
    pub fn resume(&mut self, resumable_ref: ResumableRef) -> Result<RunState, RuntimeError> {
        self.store.resume(resumable_ref)
    }

    /// calls its argument `f` with a mutable reference of the fuel of the respective [`ResumableRef`].
    ///
    /// Fuel is stored as an [`Option<u32>`], where `None` means that fuel is disabled and `Some(x)` means that `x` units of fuel is left.
    /// A ubiquitious use of this method would be using `f` to read or mutate the current fuel amount of the respective [`ResumableRef`].
    /// # Example
    /// ```
    /// use wasm::{resumable::RunState, validate, RuntimeInstance};
    /// let wasm = [ 0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
    ///             0x01, 0x04, 0x01, 0x60, 0x00, 0x00, 0x03, 0x02,
    ///             0x01, 0x00, 0x07, 0x09, 0x01, 0x05, 0x6c, 0x6f,
    ///             0x6f, 0x70, 0x73, 0x00, 0x00, 0x0a, 0x09, 0x01,
    ///             0x07, 0x00, 0x03, 0x40, 0x0c, 0x00, 0x0b, 0x0b ];
    /// // a simple module with a single function looping forever
    /// let mut instance = RuntimeInstance::new_named((), "module", &validate(&wasm).unwrap()).unwrap();
    /// let func_ref = instance.get_function_by_name("module", "loops").unwrap();
    /// let mut resumable_ref = instance.create_resumable(&func_ref, vec![], 0).unwrap();
    /// instance.access_fuel_mut(&mut resumable_ref, |x| { assert_eq!(*x, Some(0)); *x = None; }).unwrap();
    /// ```
    pub fn access_fuel_mut<R>(
        &mut self,
        resumable_ref: &mut ResumableRef,
        f: impl FnOnce(&mut Option<u32>) -> R,
    ) -> Result<R, RuntimeError> {
        self.store.access_fuel_mut(resumable_ref, f)
    }

    /// Adds a host function under module namespace `module_name` with name `name`.
    /// roughly similar to `func_alloc` in <https://webassembly.github.io/spec/core/appendix/embedding.html#functions>
    /// except the host function is made visible to other modules through these names.
    pub fn add_host_function_typed<Params: InteropValueList, Returns: InteropValueList>(
        &mut self,
        module_name: &str,
        name: &str,
        host_func: fn(&mut T, Vec<Value>) -> Vec<Value>,
    ) -> Result<FunctionRef, RuntimeError> {
        let host_func_ty = FuncType {
            params: ResultType {
                valtypes: Vec::from(Params::TYS),
            },
            returns: ResultType {
                valtypes: Vec::from(Returns::TYS),
            },
        };
        self.add_host_function(module_name, name, host_func_ty, host_func)
    }

    pub fn add_host_function(
        &mut self,
        module_name: &str,
        name: &str,
        host_func_ty: FuncType,
        host_func: fn(&mut T, Vec<Value>) -> Vec<Value>,
    ) -> Result<FunctionRef, RuntimeError> {
        let func_addr = self.store.alloc_host_func(host_func_ty, host_func);
        self.store.registry.register(
            module_name.to_owned().into(),
            name.to_owned().into(),
            store::ExternVal::Func(func_addr),
        )?;
        Ok(FunctionRef { func_addr })
    }

    pub fn user_data(&self) -> &T {
        &self.store.user_data
    }

    pub fn user_data_mut(&mut self) -> &mut T {
        &mut self.store.user_data
    }

    /// Returns the global type of some global instance by its addr.
    pub fn global_type(&self, global_addr: usize) -> GlobalType {
        self.store.global_type(global_addr)
    }

    /// Returns the current value of some global instance by its addr.
    pub fn global_read(&self, global_addr: usize) -> Value {
        self.store.global_read(global_addr)
    }

    /// Sets a new value of some global instance by its addr.
    ///
    /// # Errors
    /// - [`RuntimeError::WriteOnImmutableGlobal`]
    /// - [`RuntimeError::GlobalTypeMismatch`]
    pub fn global_write(&mut self, global_addr: usize, val: Value) -> Result<(), RuntimeError> {
        self.store.global_write(global_addr, val)
    }
}

/// Helper function to quickly construct host functions without worrying about wasm to Rust
/// type conversion. For reading/writing user data into the current configuration, simply move
/// `user_data` into the passed closure.
/// # Example
/// ```
/// use wasm::{validate, RuntimeInstance, host_function_wrapper, Value};
/// fn my_wrapped_host_func(user_data: &mut (), params: Vec<Value>) -> Vec<Value> {
///     host_function_wrapper(params, |(x, y): (u32, i32)| -> u32 {
///         let _user_data = user_data;
///         x + (y as u32)
///  })
/// }
/// fn main() {
///     let mut instance = RuntimeInstance::new(());
///     let foo_bar = instance.add_host_function_typed::<(u32,i32),u32>("foo", "bar", my_wrapped_host_func).unwrap();
/// }
/// ```
pub fn host_function_wrapper<Params: InteropValueList, Results: InteropValueList>(
    params: Vec<Value>,
    f: impl FnOnce(Params) -> Results,
) -> Vec<Value> {
    let params =
        Params::try_from_values(params.into_iter()).expect("Params match the actual parameters");
    let results = f(params);
    results.into_values()
}
