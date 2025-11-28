/// A simple function to add two numbers and return the result, using [invoke_dynamic](wasm::RuntimeInstance::invoke_dynamic)
/// instead of [invoke_named](wasm::RuntimeInstance::invoke_named).
#[test_log::test]
fn dynamic_add() {
    use wasm::{validate, Store, Value};

    let wat = r#"
    (module
        (func (export "add") (param $x i32) (param $y i32) (result i32)
            local.get $x
            local.get $y
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

    let add = store
        .instance_export_unchecked(module, "add")
        .unwrap()
        .as_func()
        .unwrap();

    let res = store
        .invoke_without_fuel_unchecked(add, vec![Value::I32(11), Value::I32(1)])
        .expect("invocation failed");
    assert_eq!(vec![Value::I32(12)], res);

    let res = store
        .invoke_without_fuel_unchecked(add, vec![Value::I32(-6i32 as u32), Value::I32(1)])
        .expect("invocation failed");
    assert_eq!(vec![Value::I32(-5i32 as u32)], res);
}
