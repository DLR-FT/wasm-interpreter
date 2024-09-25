use wasm::{validate, RuntimeInstance};

const WAT_SUBTRACT_TEMPLATE: &'static str = r#"
    (module
        (func (export "subtract") (param $x {{TYPE}}) (param $y {{TYPE}}) (result {{TYPE}})
            local.get $x
            local.get $y
            {{TYPE}}.sub
        )
    )
"#;

/// A simple function to multiply by 3 a i64 value and return the result
#[test_log::test]
pub fn i64_subtract() {
    let wat = String::from(WAT_SUBTRACT_TEMPLATE).replace("{{TYPE}}", "i64");

    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");

    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        -10 as i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1 as i64, 11 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        0 as i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0 as i64, 0 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        10 as i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-10 as i64, -20 as i64)
            )
            .unwrap()
    );

    assert_eq!(
        i64::MAX - 1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (i64::MAX - 1, 0 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        i64::MIN + 3,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (i64::MIN + 3, 0 as i64)
            )
            .unwrap()
    );
}
