/// A simple function to multiply by 2 a i32 value and return the result
#[test_log::test]
pub fn multiply() {
    use wasm::{validate, RuntimeInstance};

    let wat = r#"
    (module
        (func (export "multiply") (param $x i32) (result i32)
            local.get $x
            i32.const 3
            i32.mul)
    )
    "#;

    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");

    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(33, instance.invoke_named("multiply", 11).unwrap());
    assert_eq!(0, instance.invoke_named("multiply", 0).unwrap());
    assert_eq!(-30, instance.invoke_named("multiply", -10).unwrap());

    assert_eq!(
        i32::MAX - 5,
        instance.invoke_named("multiply", i32::MAX - 1).unwrap()
    );
    assert_eq!(
        i32::MIN + 3,
        instance.invoke_named("multiply", i32::MIN + 1).unwrap()
    );
}
