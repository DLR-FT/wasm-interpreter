pub use crate::common::wasmtime_runner::WASMTimeRunner;
pub use crate::common::*;

/// A simple function to multiply by 2 a i32 value and return the result
#[test_log::test]
pub fn multiply() {
    use wasm::{validate, RuntimeInstance};

    let wat = r#"
    (module
        (func (export "multiply") (param $x i32) (result i32)
            local.get $x
            i32.const 3
            i32.mul)
    )
    "#;
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");
    let wasmtime_instance =
        WASMTimeRunner::new(wat, ()).expect("wasmtime runner failed to instantiate");

    let mut runners = [
        FunctionRunner::new(instance.into(), 0, "multiply"),
        FunctionRunner::new(wasmtime_instance.into(), 0, "multiply"),
    ];

    poly_test(11, 33, &mut runners);
    poly_test(0, 0, &mut runners);
    poly_test(-10, -30, &mut runners);

    poly_test(i32::MAX - 1, i32::MAX - 5, &mut runners);
    poly_test(i32::MIN + 1, i32::MIN + 3, &mut runners);
}
