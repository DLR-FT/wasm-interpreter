mod common;
pub use common::wasmtime_runner::WASMTimeRunner;
pub use common::*;

/// Two simple methods for storing and loading an i32 from the first slot in linear memory.
#[test_log::test]
fn basic_memory() {
    use wasm::{validate, RuntimeInstance};

    let wat = r#"
    (module
        (memory 1)
        (func (export "store_num") (param $x i32)
            i32.const 0
            local.get $x
            i32.store)
        (func (export "load_num") (result i32)
            i32.const 0
            i32.load)
    )
    "#;

    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");
    let wasmtime_instance =
        WASMTimeRunner::new(wat, ()).expect("wasmtime runner failed to instantiate");

    let mut runners = [instance.into(), wasmtime_instance.into()];

    poly_test_once(42, (), 0, "store_num", &mut runners);
    poly_test_once((), 42, 1, "load_num", &mut runners);
}
