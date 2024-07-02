pub(crate) mod runner;
pub(crate) mod wasmtime_runner;

pub use runner::*;

extern crate wasm;
use std::fmt::Debug;
use wasm::value::InteropValueList;
use wasmtime::{WasmParams, WasmResults};

/// Test a function with all [FunctionRunner]s. `poly_test` assumes that all the runners share the same function.
///
/// # Panics
///
/// Panics if the output of any runner does not match the expected result or if any runner fails to execute.
pub fn poly_test<Params, Output, StoreType>(
    params: Params,
    expected_result: Output,
    runners: &mut [FunctionRunner<StoreType>],
) where
    Params: UniversalParams + Clone,
    Output: UniversalResults + Debug + PartialEq,
{
    for runner in runners {
        let output = runner
            .execute::<Params, Output>(params.clone())
            .expect("Runner execution failed");

        assert_eq!(output, expected_result);
    }
}

/// Test any function with all [Runner]s.
///
/// # Panics
///
/// Panics if the output of any runner does not match the expected result or if any runner fails to execute.
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
