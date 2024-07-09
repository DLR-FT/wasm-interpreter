use wasm::{validate, RuntimeInstance};

const MULTIPLY_WAT_TEMPLATE: &'static str = r#"
    (module
        (func (export "add_one") (param $x {{TYPE}}) (result {{TYPE}})
            local.get $x
            {{TYPE}}.const 1
            {{TYPE}}.add)
    )
"#;

/// A simple function to add 1 to an i32 and return the result
#[test_log::test]
fn i32_add_one() {
    let wat = String::from(MULTIPLY_WAT_TEMPLATE).replace("{{TYPE}}", "i32");
    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(12, instance.invoke_named("add_one", 11).unwrap());
    assert_eq!(1, instance.invoke_named("add_one", 0).unwrap());
    assert_eq!(-5, instance.invoke_named("add_one", -6).unwrap());
}

/// A simple function to add 1 to an i64 and return the result
#[test_log::test]
fn i64_add_one() {
    let wat = String::from(MULTIPLY_WAT_TEMPLATE).replace("{{TYPE}}", "i64");
    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(12 as i64, instance.invoke_func(0, 11 as i64).unwrap());
    assert_eq!(1 as i64, instance.invoke_func(0, 0 as i64).unwrap());
    assert_eq!(-5 as i64, instance.invoke_func(0, -6 as i64).unwrap());
}
