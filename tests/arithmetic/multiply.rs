use wasm::{validate, RuntimeInstance};

const MULTIPLY_WAT_TEMPLATE: &'static str = r#"
    (module
        (func (export "multiply") (param $x {{TYPE}}) (result {{TYPE}})
            local.get $x
            {{TYPE}}.const 3
            {{TYPE}}.mul
        )
    )
"#;
/// A simple function to multiply by 3 a i32 value and return the result
#[test_log::test]
pub fn i32_multiply() {
    let wat = String::from(MULTIPLY_WAT_TEMPLATE).replace("{{TYPE}}", "i32");

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

/// A simple function to multiply by 3 a i64 value and return the result
#[test_log::test]
pub fn i64_multiply() {
    let wat = String::from(MULTIPLY_WAT_TEMPLATE).replace("{{TYPE}}", "i64");

    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");

    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(33 as i64, instance.invoke_func(0, 11 as i64).unwrap());
    assert_eq!(0 as i64, instance.invoke_func(0, 0 as i64).unwrap());
    assert_eq!(-30 as i64, instance.invoke_func(0, -10 as i64).unwrap());

    assert_eq!(i64::MAX - 5, instance.invoke_func(0, i64::MAX - 1).unwrap());
    assert_eq!(i64::MIN + 3, instance.invoke_func(0, i64::MIN + 1).unwrap());
}
