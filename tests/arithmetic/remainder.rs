use wasm::{validate, RuntimeInstance};
use wasm::{RuntimeError, DEFAULT_MODULE};
const REM_S_WAT: &'static str = r#"
    (module
        (func (export "rem_s") (param $divisor {{TYPE}}) (param $dividend {{TYPE}}) (result {{TYPE}})
            local.get $divisor
            local.get $dividend
            {{TYPE}}.rem_s)
    )
"#;

const REM_U_WAT: &'static str = r#"
    (module
        (func (export "rem_u") (param $divisor {{TYPE}}) (param $dividend {{TYPE}}) (result {{TYPE}})
            local.get $divisor
            local.get $dividend
            {{TYPE}}.rem_u)
    )
"#;

/// A simple function to test i64 signed remainder
#[test_log::test]
pub fn i64_remainder_signed_simple() {
    let wat = String::from(REM_S_WAT).replace("{{TYPE}}", "i64");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        0 as i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (20 as i64, 2 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        999 as i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (10_000 as i64, 9_001 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        -2 as i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-20 as i64, 3 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        -2 as i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-20 as i64, -3 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        2 as i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (20 as i64, -3 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        2 as i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (20 as i64, 3 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        0 as i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (i64::MIN as i64, -1 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        0 as i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (i64::MIN as i64, 2 as i64)
            )
            .unwrap()
    );
}

/// A simple function to test i64 signed remainder's RuntimeError when dividing by 0
#[test_log::test]
pub fn i64_remainder_signed_panic_dividend_0() {
    let wat = String::from(REM_S_WAT).replace("{{TYPE}}", "i64");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let result = instance.invoke::<(i64, i64), i64>(
        &instance.get_function_by_index(0, 0).unwrap(),
        (222 as i64, 0 as i64),
    );

    assert_eq!(result.unwrap_err(), RuntimeError::DivideBy0);
}

/// A simple function to test i64 unsigned remainder
#[test_log::test]
pub fn i64_remainder_unsigned_simple() {
    let wat = String::from(REM_U_WAT).replace("{{TYPE}}", "i64");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        0 as i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (i64::MIN, 2 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        i64::MIN,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (i64::MIN, -2 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        (i64::MAX - 1),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-2 as i64, i64::MIN)
            )
            .unwrap()
    );
    assert_eq!(
        2 as i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (2 as i64, i64::MIN)
            )
            .unwrap()
    );

    assert_eq!(
        0 as i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (20 as i64, 2 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        999 as i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (10_000 as i64, 9_001 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        2 as i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-20 as i64, 3 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        -20 as i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-20 as i64, -3 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        20 as i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (20 as i64, -3 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        2 as i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (20 as i64, 3 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        i64::MIN,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (i64::MIN, -1 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        0 as i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (i64::MIN, 2 as i64)
            )
            .unwrap()
    );
}

/// A simple function to test i64 signed remainder's RuntimeError when dividing by 0
#[test_log::test]
pub fn i64_remainder_unsigned_panic_dividend_0() {
    let wat = String::from(REM_U_WAT).replace("{{TYPE}}", "i64");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let result = instance
        .invoke::<(i64, i64), i64>(&instance.get_function_by_index(0, 0).unwrap(), (222, 0));

    assert_eq!(result.unwrap_err(), RuntimeError::DivideBy0);
}

/// A simple function to test signed remainder
#[test_log::test]
pub fn i32_remainder_signed_simple() {
    let wat = String::from(REM_S_WAT).replace("{{TYPE}}", "i32");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        0,
        instance
            .invoke(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "rem_s")
                    .unwrap(),
                (20, 2)
            )
            .unwrap()
    );
    assert_eq!(
        999,
        instance
            .invoke(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "rem_s")
                    .unwrap(),
                (10_000, 9_001)
            )
            .unwrap()
    );
    assert_eq!(
        -2,
        instance
            .invoke(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "rem_s")
                    .unwrap(),
                (-20, 3)
            )
            .unwrap()
    );
    assert_eq!(
        -2,
        instance
            .invoke(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "rem_s")
                    .unwrap(),
                (-20, -3)
            )
            .unwrap()
    );
    assert_eq!(
        2,
        instance
            .invoke(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "rem_s")
                    .unwrap(),
                (20, -3)
            )
            .unwrap()
    );
    assert_eq!(
        2,
        instance
            .invoke(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "rem_s")
                    .unwrap(),
                (20, 3)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "rem_s")
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
                    .get_function_by_name(DEFAULT_MODULE, "rem_s")
                    .unwrap(),
                (i32::MIN, 2)
            )
            .unwrap()
    );
}

/// A simple function to test signed remainder's RuntimeError when dividing by 0
#[test_log::test]
pub fn remainder_signed_panic_dividend_0() {
    let wat = String::from(REM_S_WAT).replace("{{TYPE}}", "i32");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let result = instance.invoke::<(i32, i32), i32>(
        &instance
            .get_function_by_name(DEFAULT_MODULE, "rem_s")
            .unwrap(),
        (222, 0),
    );

    assert_eq!(result.unwrap_err(), RuntimeError::DivideBy0);
}

/// A simple function to test unsigned remainder
#[test_log::test]
pub fn i32_remainder_unsigned_simple() {
    let wat = String::from(REM_U_WAT).replace("{{TYPE}}", "i32");
    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        0,
        instance
            .invoke(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "rem_u")
                    .unwrap(),
                (i32::MIN, 2)
            )
            .unwrap()
    );
    assert_eq!(
        i32::MIN,
        instance
            .invoke(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "rem_u")
                    .unwrap(),
                (i32::MIN, -2)
            )
            .unwrap()
    );
    assert_eq!(
        (i32::MIN + 2) * (-1),
        instance
            .invoke(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "rem_u")
                    .unwrap(),
                (-2, i32::MIN)
            )
            .unwrap()
    );
    assert_eq!(
        2,
        instance
            .invoke(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "rem_u")
                    .unwrap(),
                (2, i32::MIN)
            )
            .unwrap()
    );
    assert_eq!(
        i32::MAX,
        instance
            .invoke(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "rem_u")
                    .unwrap(),
                (i32::MAX, i32::MIN)
            )
            .unwrap()
    );

    assert_eq!(
        0,
        instance
            .invoke(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "rem_u")
                    .unwrap(),
                (20, 2)
            )
            .unwrap()
    );
    assert_eq!(
        999,
        instance
            .invoke(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "rem_u")
                    .unwrap(),
                (10_000, 9_001)
            )
            .unwrap()
    );
    assert_eq!(
        2,
        instance
            .invoke(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "rem_u")
                    .unwrap(),
                (-20, 3)
            )
            .unwrap()
    );
    assert_eq!(
        -20,
        instance
            .invoke(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "rem_u")
                    .unwrap(),
                (-20, -3)
            )
            .unwrap()
    );
    assert_eq!(
        20,
        instance
            .invoke(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "rem_u")
                    .unwrap(),
                (20, -3)
            )
            .unwrap()
    );
    assert_eq!(
        2,
        instance
            .invoke(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "rem_u")
                    .unwrap(),
                (20, 3)
            )
            .unwrap()
    );
    assert_eq!(
        i32::MIN,
        instance
            .invoke(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "rem_u")
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
                    .get_function_by_name(DEFAULT_MODULE, "rem_u")
                    .unwrap(),
                (i32::MIN, 2)
            )
            .unwrap()
    );
}

/// A simple function to test signed remainder's RuntimeError when dividing by 0
#[test_log::test]
pub fn i32_remainder_unsigned_panic_dividend_0() {
    let wat = String::from(REM_U_WAT).replace("{{TYPE}}", "i32");
    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let result = instance.invoke::<(i32, i32), i32>(
        &instance
            .get_function_by_name(DEFAULT_MODULE, "rem_u")
            .unwrap(),
        (222, 0),
    );

    assert_eq!(result.unwrap_err(), RuntimeError::DivideBy0);
}
