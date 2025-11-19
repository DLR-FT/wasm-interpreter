//! The WASM program stores 42 into linear memory upon instantiation through a start function.
//! Then it reads the same value and checks its value.

use wasm::{validate, RuntimeInstance};

#[test_log::test]
fn start_function() {
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
    let mut instance = RuntimeInstance::new(());
    let module = instance
        .store
        .module_instantiate(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;

    let load_num = instance
        .store
        .instance_export(module, "load_num")
        .unwrap()
        .as_func()
        .unwrap();

    assert_eq!(
        42,
        instance
            .store
            .invoke_typed_without_fuel(load_num, ())
            .unwrap()
    );
}
