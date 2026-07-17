//! The WASM program stores 42 into linear memory upon instantiation through a start function.
//! Then it reads the same value and checks its value.

use dlr_wasm_interpreter::decode_and_validate;
use dlr_wasm_interpreter_checked::Store;

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

    let module = decode_and_validate(&wasm_bytes, &mut ()).expect("validation failed");
    let mut store = Store::new(());
    let module = store
        .module_instantiate(&module, Vec::new(), None)
        .unwrap()
        .module_addr;

    let load_num = store
        .instance_export(module, "load_num")
        .unwrap()
        .as_func()
        .unwrap();

    assert_eq!(42, store.invoke_simple_typed(load_num, ()).unwrap());
}
