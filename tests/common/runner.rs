use crate::common::wasmtime_runner;
use std::error::Error;

extern crate wasm;

/// To utilize the `Runner` trait, all parameters and results must be able to interact with the WASM runners.
/// In this case, the `UniversalParams` and `UniversalResults` traits are used to define the necessary constraints.
pub trait UniversalParams: wasmtime::WasmParams + wasm::value::InteropValueList {}
impl<T> UniversalParams for T where T: wasmtime::WasmParams + wasm::value::InteropValueList {}

/// To utilize the `Runner` trait, all parameters and results must be able to interact with the WASM runners.
/// In this case, the `UniversalParams` and `UniversalResults` traits are used to define the necessary constraints.
pub trait UniversalResults: wasmtime::WasmResults + wasm::value::InteropValueList {}
impl<T> UniversalResults for T where T: wasmtime::WasmResults + wasm::value::InteropValueList {}

// .--------------.
// |    Runner    |
// '--------------'

// TODO: also add wasmi?

/// The `Runner` enum is used to abstract over the different type WASM runners.
///
/// The generic `StoreType` is used to define what type is handled in the runners' [Store].
///
/// A `Runner` must live as long as the WASM instance it is associated with.
pub enum Runner<'a, StoreType> {
    Interpreter(wasm::RuntimeInstance<'a>),
    WASMTime(wasmtime_runner::WASMTimeRunner<StoreType>),
}

impl<StoreType> Runner<'_, StoreType> {
    /// Executes a function with the given parameters and returns the result.
    ///
    /// # Undefined behavior
    ///
    /// This function implicitly assumes that the function ID and function name are linked to the same function.
    /// If the function ID and function name are not linked to the same function, the behavior is undefined (though
    /// it will likely result in the wrong function being executed).
    pub fn execute<Params: UniversalParams, Output: UniversalResults>(
        &mut self,
        func_id: usize,
        func_name: &str,
        params: Params,
    ) -> Result<Output, Box<dyn Error>> {
        match self {
            Runner::Interpreter(runner) => Ok(runner.invoke_func(func_id, params)),
            Runner::WASMTime(runner) => Ok(runner.execute(func_name, params)?),
        }
    }
}

// .-----------------------------------.
// | `From` implementations for Runner |
// '-----------------------------------'

impl<'a, StoreType> From<wasm::RuntimeInstance<'a>> for Runner<'a, StoreType> {
    fn from(value: wasm::RuntimeInstance<'a>) -> Self {
        Runner::Interpreter(value)
    }
}

impl<'a, StoreType> From<wasmtime_runner::WASMTimeRunner<StoreType>> for Runner<'a, StoreType> {
    fn from(value: wasmtime_runner::WASMTimeRunner<StoreType>) -> Self {
        Runner::WASMTime(value)
    }
}

// .-----------------.
// | Function Runner |
// '-----------------'

/// The `FunctionRunner` struct is a wrapper around a `Runner` that is used to execute a specific function.
pub struct FunctionRunner<'a, StoreType> {
    inner: Runner<'a, StoreType>,
    func_id: usize,
    func_name: &'a str,
}

impl<'a, StoreType> FunctionRunner<'a, StoreType> {
    /// Creates a new `FunctionRunner` with the given `Runner`, function ID, and function name.
    ///
    /// # Undefined behavior
    ///
    /// This function implicitly assumes that the function ID and function name are linked to the same function.
    /// If the function ID and function name are not linked to the same function, the behavior is undefined (though
    /// it will likely result in the wrong function being executed).
    pub fn new(inner: Runner<'a, StoreType>, func_id: usize, func_name: &'a str) -> Self {
        FunctionRunner {
            inner,
            func_id,
            func_name,
        }
    }

    pub fn execute<Params: UniversalParams, Output: UniversalResults>(
        &mut self,
        params: Params,
    ) -> Result<Output, Box<dyn Error>> {
        self.inner.execute(self.func_id, self.func_name, params)
    }
}
