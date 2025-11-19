use wasm::{validate, RuntimeInstance};

const MULTIPLY_WAT_TEMPLATE: &str = r#"
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

    let mut instance = RuntimeInstance::new(());
    let module = instance
        .store
        .module_instantiate(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;

    let multiply = instance
        .store
        .instance_export(module, "multiply")
        .unwrap()
        .as_func()
        .unwrap();

    assert_eq!(
        33,
        instance
            .store
            .invoke_typed_without_fuel(multiply, 11)
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .store
            .invoke_typed_without_fuel(multiply, 0)
            .unwrap()
    );
    assert_eq!(
        -30,
        instance
            .store
            .invoke_typed_without_fuel(multiply, -10)
            .unwrap()
    );

    assert_eq!(
        i32::MAX - 5,
        instance
            .store
            .invoke_typed_without_fuel(multiply, i32::MAX - 1)
            .unwrap()
    );
    assert_eq!(
        i32::MIN + 3,
        instance
            .store
            .invoke_typed_without_fuel(multiply, i32::MIN + 1)
            .unwrap()
    );
}

/// A simple function to multiply by 3 a i64 value and return the result
#[test_log::test]
pub fn i64_multiply() {
    let wat = String::from(MULTIPLY_WAT_TEMPLATE).replace("{{TYPE}}", "i64");

    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");

    let mut instance = RuntimeInstance::new(());
    let module = instance
        .store
        .module_instantiate(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;

    let multiply = instance
        .store
        .instance_export(module, "multiply")
        .unwrap()
        .as_func()
        .unwrap();

    assert_eq!(
        33_i64,
        instance
            .store
            .invoke_typed_without_fuel(multiply, 11_i64)
            .unwrap()
    );
    assert_eq!(
        0_i64,
        instance
            .store
            .invoke_typed_without_fuel(multiply, 0_i64)
            .unwrap()
    );
    assert_eq!(
        -30_i64,
        instance
            .store
            .invoke_typed_without_fuel(multiply, -10_i64)
            .unwrap()
    );

    assert_eq!(
        i64::MAX - 5,
        instance
            .store
            .invoke_typed_without_fuel(multiply, i64::MAX - 1)
            .unwrap()
    );
    assert_eq!(
        i64::MIN + 3,
        instance
            .store
            .invoke_typed_without_fuel(multiply, i64::MIN + 1)
            .unwrap()
    );
}
