pub(crate) mod wasmtime_runner;

extern crate wasm;
use std::error::Error;
use std::fmt::Debug;
use wasm::value::InteropValueList;
use wasmtime::{WasmParams, WasmResults, WasmTy};

// TODO: also add wasmi?
pub enum Runner<'a, StoreType> {
    Interpreter(wasm::RuntimeInstance<'a>),
    WASMTime(wasmtime_runner::WASMTimeRunner<StoreType>),
}

impl<StoreType> Runner<'_, StoreType> {
    pub fn execute<Params: InteropValueList + WasmParams, Output: InteropValueList + WasmResults>(
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

pub struct FunctionRunner<'a, StoreType> {
    inner: Runner<'a, StoreType>,
    func_id: usize,
    func_name: &'a str,
}

impl<'a, StoreType> FunctionRunner<'a, StoreType> {
    pub fn new(inner: Runner<'a, StoreType>, func_id: usize, func_name: &'a str) -> Self {
        FunctionRunner {
            inner,
            func_id,
            func_name,
        }
    }

    pub fn execute<Params: InteropValueList + WasmParams, Output: InteropValueList + WasmResults>(
        &mut self,
        params: Params,
    ) -> Result<Output, Box<dyn Error>> {
        self.inner.execute(self.func_id, self.func_name, params)
    }
}

pub fn poly_test<Params, Output, StoreType>(
    params: Params,
    expected_result: Output,
    runners: &mut [FunctionRunner<StoreType>],
) where
    Params: InteropValueList + WasmTy + Clone,
    Output: InteropValueList + WasmTy + Debug + PartialEq,
{
    for runner in runners {
        let output = runner
            .execute::<Params, Output>(params.clone())
            .expect("Runner execution failed");

        assert_eq!(output, expected_result);
    }
}

pub fn poly_test_once<Params, Output, StoreType>(
    params: Params,
    expected_result: Output,
    function_id: usize,
    function_name: &str,
    runners: &mut [Runner<StoreType>],
) where
    Params: InteropValueList + WasmParams + Clone,
    Output: InteropValueList + WasmResults + Debug + PartialEq,
{
    for runner in runners {
        let output = runner
            .execute::<Params, Output>(function_id, function_name, params.clone())
            .expect("Runner execution failed");

        assert_eq!(output, expected_result);
    }
}