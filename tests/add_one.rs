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
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(12, instance.invoke_func(0, 11));
    assert_eq!(1, instance.invoke_func(0, 0));
    assert_eq!(-5, instance.invoke_func(0, -6));
}
