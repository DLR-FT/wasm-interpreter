use dlr_wasm_interpreter::decode_and_validate;
use dlr_wasm_interpreter_checked::Store;

/// A simple function to add 2 two i32s but using the RETURN opcode.
#[test_log::test]
fn return_valid() {
    let wat = r#"
    (module
        (func (export "add") (param $x i32) (param $y i32) (result i32)
            local.get $x
            local.get $x
            local.get $x
            local.get $x
            local.get $x
            local.get $x
            local.get $x
            local.get $y
            i32.add
            return
        )
    )
    "#;
    let wasm_bytes = wat::parse_str(wat).unwrap();

    let module = decode_and_validate(&wasm_bytes, &mut ()).expect("validation failed");
    let mut store = Store::new(());
    let module = store
        .module_instantiate(&module, Vec::new(), None)
        .unwrap()
        .module_addr;

    let add = store
        .instance_export(module, "add")
        .unwrap()
        .as_func()
        .unwrap();

    assert_eq!(12, store.invoke_simple_typed(add, (10, 2)).unwrap());
    assert_eq!(2, store.invoke_simple_typed(add, (0, 2)).unwrap());
    assert_eq!(-4, store.invoke_simple_typed(add, (-6, 2)).unwrap());
}
