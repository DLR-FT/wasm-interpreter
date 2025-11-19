use wasm::{validate, RuntimeInstance};

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

    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(());
    let module = instance
        .store
        .module_instantiate(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;

    let add = instance
        .store
        .instance_export(module, "add")
        .unwrap()
        .as_func()
        .unwrap();

    assert_eq!(
        12,
        instance
            .store
            .invoke_typed_without_fuel(add, (10, 2))
            .unwrap()
    );
    assert_eq!(
        2,
        instance
            .store
            .invoke_typed_without_fuel(add, (0, 2))
            .unwrap()
    );
    assert_eq!(
        -4,
        instance
            .store
            .invoke_typed_without_fuel(add, (-6, 2))
            .unwrap()
    );
}
