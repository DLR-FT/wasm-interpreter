use wasm::{validate, Store};

/// This test checks if we can validate and executa a module which has two functions with the same signature.
#[test_log::test]
fn same_type_fn() {
    let wat = r#"
    (module
        (func (export "add_one") (param $x i32) (result i32)
            local.get $x
            i32.const 1
            i32.add)

        (func (export "add_two") (param $x i32) (result i32)
            local.get $x
            i32.const 2
            i32.add)
    )
    "#;
    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut store = Store::new(());
    let module = store
        .module_instantiate_unchecked(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;

    let add_one = store
        .instance_export_unchecked(module, "add_one")
        .unwrap()
        .as_func()
        .unwrap();
    let add_two = store
        .instance_export_unchecked(module, "add_two")
        .unwrap()
        .as_func()
        .unwrap();

    assert_eq!(
        -5,
        store
            .invoke_typed_without_fuel_unchecked(add_one, -6)
            .unwrap()
    );
    assert_eq!(
        -4,
        store
            .invoke_typed_without_fuel_unchecked(add_two, -6)
            .unwrap()
    );
}
