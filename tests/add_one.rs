use wasm::{validate, RuntimeInstance};

const MULTIPLY_WAT_TEMPLATE: &str = r#"
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
    let (mut instance, module) = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");

    let add_one = instance
        .store
        .instance_export(module, "add_one")
        .unwrap()
        .as_func()
        .unwrap();

    assert_eq!(
        12,
        instance
            .store
            .invoke_typed_without_fuel(add_one, 11)
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .store
            .invoke_typed_without_fuel(add_one, 0)
            .unwrap()
    );
    assert_eq!(
        -5,
        instance
            .store
            .invoke_typed_without_fuel(add_one, -6)
            .unwrap()
    );
}

/// A simple function to add 1 to an i64 and return the result
#[test_log::test]
fn i64_add_one() {
    let wat = String::from(MULTIPLY_WAT_TEMPLATE).replace("{{TYPE}}", "i64");
    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let (mut instance, module) = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");

    let add_one = instance
        .store
        .instance_export(module, "add_one")
        .unwrap()
        .as_func()
        .unwrap();

    assert_eq!(
        12_i64,
        instance
            .store
            .invoke_typed_without_fuel(add_one, 11_i64)
            .unwrap()
    );
    assert_eq!(
        1_i64,
        instance
            .store
            .invoke_typed_without_fuel(add_one, 0_i64)
            .unwrap()
    );
    assert_eq!(
        -5_i64,
        instance
            .store
            .invoke_typed_without_fuel(add_one, -6_i64)
            .unwrap()
    );
}
