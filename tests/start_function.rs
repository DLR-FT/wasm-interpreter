mod common;
pub use common::wasmtime_runner::WASMTimeRunner;
pub use common::*;

/// The WASM program stores 42 into linear memory upon instantiation through a start function.
/// Then it reads the same value and checks its value.
#[test_log::test]
fn start_function() {
    use wasm::{validate, RuntimeInstance};

    let wat = r#"
    (module
        (memory 1)

        (func $store42
            i32.const 0
            i32.const 42
            i32.store)

        (start $store42)

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

    poly_test_once((), 42, 1, "load_num", &mut runners);
}
