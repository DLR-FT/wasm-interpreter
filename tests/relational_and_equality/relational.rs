use wasm::{validate, RuntimeInstance};
const BASE_WAT: &'static str = r#"
    (module
        (func (export "template") (param $x i32) (param $y i32) (result i32)
            local.get $x
            local.get $y
            i32.{{0}})
    )
    "#;

/// A simple function to test the i32.lt_s function
#[test_log::test]
pub fn i32_lt_s() {
    let wat = String::from(BASE_WAT).replace("{{0}}", "lt_s");

    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");

    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(0, instance.invoke_func(0, (0, 0)).unwrap());
    assert_eq!(1, instance.invoke_func(0, (0, 9001)).unwrap());
}

/// A simple function to test the i32.lt_u function
#[test_log::test]
pub fn i32_lt_u() {
    let wat = String::from(BASE_WAT).replace("{{0}}", "lt_u");

    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");

    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(0, instance.invoke_func(0, (0, 0)).unwrap());
    assert_eq!(1, instance.invoke_func(0, (0, 9001)).unwrap());
}
