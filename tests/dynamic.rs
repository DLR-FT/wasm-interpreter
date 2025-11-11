/// A simple function to add two numbers and return the result, using [invoke_dynamic](wasm::RuntimeInstance::invoke_dynamic)
/// instead of [invoke_named](wasm::RuntimeInstance::invoke_named).
#[test_log::test]
fn dynamic_add() {
    use wasm::{validate, RuntimeInstance, Value};

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
    let (mut instance, module) = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");

    let func = instance.get_function_by_index(module, 0).unwrap();

    let res = instance
        .invoke(func, vec![Value::I32(11), Value::I32(1)])
        .expect("invocation failed");
    assert_eq!(vec![Value::I32(12)], res);

    let res = instance
        .invoke(func, vec![Value::I32(-6i32 as u32), Value::I32(1)])
        .expect("invocation failed");
    assert_eq!(vec![Value::I32(-5i32 as u32)], res);
}
