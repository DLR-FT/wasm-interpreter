mod common; 
pub use common::*;
pub use common::wasmtime_runner::WASMTimeRunner;

/// A simple function to add 1 to an i32 and return the result
#[test_log::test]
fn add_one() {
    use wasm::{validate, RuntimeInstance};

    let wat = r#"
    (module
        (func (export "add_one") (param $x i32) (result i32)
            local.get $x
            i32.const 1
            i32.add)
    )
    "#;
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");
    let wasmtime_instance = WASMTimeRunner::new(wat, ()).expect("wasmtime runner failed to instantiate");

    let mut runners = [
        FunctionRunner::new(instance.into(), 0, "add_one"),
        FunctionRunner::new(wasmtime_instance.into(), 0, "add_one")
    ];

    poly_test(11, 12, &mut runners);
    poly_test(0, 1, &mut runners);
    poly_test(-5, -4, &mut runners);
}
