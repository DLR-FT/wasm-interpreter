use wasm::{validate, RuntimeInstance};
use wasm::{RuntimeError, DEFAULT_MODULE};

const WAT_SIGNED_DIVISION_TEMPLATE: &str = r#"
    (module
        (func (export "signed_division") (param $divisor {{TYPE}}) (param $dividend {{TYPE}}) (result {{TYPE}})
            local.get $divisor
            local.get $dividend
            {{TYPE}}.div_s)
    )
"#;

const WAT_UNSIGNED_DIVISION_TEMPLATE: &str = r#"
    (module
        (func (export "unsigned_division") (param $divisor {{TYPE}}) (param $dividend {{TYPE}}) (result {{TYPE}})
            local.get $divisor
            local.get $dividend
            {{TYPE}}.div_u)
    )
"#;

/// A simple function to test signed i32 division
#[test_log::test]
pub fn i32_division_signed_simple() {
    let wat = String::from(WAT_SIGNED_DIVISION_TEMPLATE).replace("{{TYPE}}", "i32");

    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");

    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        10,
        instance
            .invoke(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "signed_division")
                    .unwrap(),
                (20, 2)
            )
            .unwrap()
    );
    assert_eq!(
        9_001,
        instance
            .invoke(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "signed_division")
                    .unwrap(),
                (81_018_001, 9_001)
            )
            .unwrap()
    );
    assert_eq!(
        -10,
        instance
            .invoke(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "signed_division")
                    .unwrap(),
                (20, -2)
            )
            .unwrap()
    );
    assert_eq!(
        10,
        instance
            .invoke(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "signed_division")
                    .unwrap(),
                (-20, -2)
            )
            .unwrap()
    );
    assert_eq!(
        -10,
        instance
            .invoke(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "signed_division")
                    .unwrap(),
                (-20, 2)
            )
            .unwrap()
    );
    assert_eq!(
        10,
        instance
            .invoke(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "signed_division")
                    .unwrap(),
                (20, 2)
            )
            .unwrap()
    );
    assert_eq!(
        9_001,
        instance
            .invoke(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "signed_division")
                    .unwrap(),
                (81_018_001, 9_001)
            )
            .unwrap()
    );
    assert_eq!(
        -10,
        instance
            .invoke(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "signed_division")
                    .unwrap(),
                (20, -2)
            )
            .unwrap()
    );
    assert_eq!(
        10,
        instance
            .invoke(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "signed_division")
                    .unwrap(),
                (-20, -2)
            )
            .unwrap()
    );
    assert_eq!(
        -10,
        instance
            .invoke(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "signed_division")
                    .unwrap(),
                (-20, 2)
            )
            .unwrap()
    );
}

/// A simple function to test i32 signed division's RuntimeError when dividing by 0
#[test_log::test]
pub fn i32_division_signed_panic_dividend_0() {
    let wat = String::from(WAT_SIGNED_DIVISION_TEMPLATE).replace("{{TYPE}}", "i32");

    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");

    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let result = instance.invoke::<(i32, i32), i32>(
        &instance
            .get_function_by_name(DEFAULT_MODULE, "signed_division")
            .unwrap(),
        (222, 0),
    );

    assert_eq!(result.unwrap_err(), RuntimeError::DivideBy0);
}

/// A simple function to test i32 signed division's RuntimeError when we are dividing the i32 minimum by -1 (which gives an unrepresentable result - overflow)
#[test_log::test]
pub fn i32_division_signed_panic_result_unrepresentable() {
    let wat = String::from(WAT_SIGNED_DIVISION_TEMPLATE).replace("{{TYPE}}", "i32");

    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");

    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let result = instance.invoke::<(i32, i32), i32>(
        &instance
            .get_function_by_name(DEFAULT_MODULE, "signed_division")
            .unwrap(),
        (i32::MIN, -1),
    );

    assert_eq!(result.unwrap_err(), RuntimeError::UnrepresentableResult);
}

/// A simple function to test i32 unsigned division
#[test_log::test]
pub fn i32_division_unsigned_simple() {
    let wat = String::from(WAT_UNSIGNED_DIVISION_TEMPLATE).replace("{{TYPE}}", "i32");

    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");

    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        10,
        instance
            .invoke(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "unsigned_division")
                    .unwrap(),
                (20, 2)
            )
            .unwrap()
    );
    assert_eq!(
        9_001,
        instance
            .invoke(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "unsigned_division")
                    .unwrap(),
                (81_018_001, 9_001)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "unsigned_division")
                    .unwrap(),
                (i32::MIN, -1)
            )
            .unwrap()
    );

    assert_eq!(
        0,
        instance
            .invoke(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "unsigned_division")
                    .unwrap(),
                (i32::MIN, -1)
            )
            .unwrap()
    );
    assert_eq!(
        -20,
        instance
            .invoke(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "unsigned_division")
                    .unwrap(),
                (-20, 1)
            )
            .unwrap()
    );
    assert_eq!(
        2147483638,
        instance
            .invoke(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "unsigned_division")
                    .unwrap(),
                (-20, 2)
            )
            .unwrap()
    );
    assert_eq!(
        1431655758,
        instance
            .invoke(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "unsigned_division")
                    .unwrap(),
                (-20, 3)
            )
            .unwrap()
    );
    assert_eq!(
        1073741819,
        instance
            .invoke(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "unsigned_division")
                    .unwrap(),
                (-20, 4)
            )
            .unwrap()
    );
}

/// A simple function to test i32 unsigned division's RuntimeError when dividing by 0
#[test_log::test]
pub fn i32_division_unsigned_panic_dividend_0() {
    let wat = String::from(WAT_UNSIGNED_DIVISION_TEMPLATE).replace("{{TYPE}}", "i32");

    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");

    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let result = instance.invoke::<(i32, i32), i32>(
        &instance
            .get_function_by_name(DEFAULT_MODULE, "unsigned_division")
            .unwrap(),
        (222, 0),
    );

    assert_eq!(result.unwrap_err(), RuntimeError::DivideBy0);
}

/// A simple function to test signed i64 division
#[test_log::test]
pub fn i64_division_signed_simple() {
    let wat = String::from(WAT_SIGNED_DIVISION_TEMPLATE).replace("{{TYPE}}", "i64");

    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");

    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        10_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (20_i64, 2_i64)
            )
            .unwrap()
    );
    assert_eq!(
        9_001_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (81_018_001_i64, 9_001_i64)
            )
            .unwrap()
    );
    assert_eq!(
        -10_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (20_i64, -2_i64)
            )
            .unwrap()
    );
    assert_eq!(
        10_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-20_i64, -2_i64)
            )
            .unwrap()
    );
    assert_eq!(
        -10_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-20_i64, 2_i64)
            )
            .unwrap()
    );
}
