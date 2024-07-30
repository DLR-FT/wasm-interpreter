use wasm::{validate, RuntimeError, RuntimeInstance};
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
        instance.invoke_func(0, (20 as i64, 2 as i64)).unwrap()
    );
    assert_eq!(
        999 as i64,
        instance
            .invoke_func(0, (10_000 as i64, 9_001 as i64))
            .unwrap()
    );
    assert_eq!(
        -2 as i64,
        instance.invoke_func(0, (-20 as i64, 3 as i64)).unwrap()
    );
    assert_eq!(
        -2 as i64,
        instance.invoke_func(0, (-20 as i64, -3 as i64)).unwrap()
    );
    assert_eq!(
        2 as i64,
        instance.invoke_func(0, (20 as i64, -3 as i64)).unwrap()
    );
    assert_eq!(
        2 as i64,
        instance.invoke_func(0, (20 as i64, 3 as i64)).unwrap()
    );
    assert_eq!(
        0 as i64,
        instance
            .invoke_func(0, (i64::MIN as i64, -1 as i64))
            .unwrap()
    );
    assert_eq!(
        0 as i64,
        instance
            .invoke_func(0, (i64::MIN as i64, 2 as i64))
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

    let result = instance.invoke_func::<(i64, i64), i64>(0, (222 as i64, 0 as i64));

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
        instance.invoke_func(0, (i64::MIN, 2 as i64)).unwrap()
    );
    assert_eq!(
        i64::MIN,
        instance.invoke_func(0, (i64::MIN, -2 as i64)).unwrap()
    );
    assert_eq!(
        (i64::MAX - 1),
        instance.invoke_func(0, (-2 as i64, i64::MIN)).unwrap()
    );
    assert_eq!(
        2 as i64,
        instance.invoke_func(0, (2 as i64, i64::MIN)).unwrap()
    );

    assert_eq!(
        0 as i64,
        instance.invoke_func(0, (20 as i64, 2 as i64)).unwrap()
    );
    assert_eq!(
        999 as i64,
        instance
            .invoke_func(0, (10_000 as i64, 9_001 as i64))
            .unwrap()
    );
    assert_eq!(
        2 as i64,
        instance.invoke_func(0, (-20 as i64, 3 as i64)).unwrap()
    );
    assert_eq!(
        -20 as i64,
        instance.invoke_func(0, (-20 as i64, -3 as i64)).unwrap()
    );
    assert_eq!(
        20 as i64,
        instance.invoke_func(0, (20 as i64, -3 as i64)).unwrap()
    );
    assert_eq!(
        2 as i64,
        instance.invoke_func(0, (20 as i64, 3 as i64)).unwrap()
    );
    assert_eq!(
        i64::MIN,
        instance.invoke_func(0, (i64::MIN, -1 as i64)).unwrap()
    );
    assert_eq!(
        0 as i64,
        instance.invoke_func(0, (i64::MIN, 2 as i64)).unwrap()
    );
}

/// A simple function to test i64 signed remainder's RuntimeError when dividing by 0
#[test_log::test]
pub fn i64_remainder_unsigned_panic_dividend_0() {
    let wat = String::from(REM_U_WAT).replace("{{TYPE}}", "i64");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let result = instance.invoke_func::<(i64, i64), i64>(0, (222, 0));

    assert_eq!(result.unwrap_err(), RuntimeError::DivideBy0);
}

/// A simple function to test signed remainder
#[test_log::test]
pub fn i32_remainder_signed_simple() {
    let wat = String::from(REM_S_WAT).replace("{{TYPE}}", "i32");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(0, instance.invoke_named("rem_s", (20, 2)).unwrap());
    assert_eq!(
        999,
        instance.invoke_named("rem_s", (10_000, 9_001)).unwrap()
    );
    assert_eq!(-2, instance.invoke_named("rem_s", (-20, 3)).unwrap());
    assert_eq!(-2, instance.invoke_named("rem_s", (-20, -3)).unwrap());
    assert_eq!(2, instance.invoke_named("rem_s", (20, -3)).unwrap());
    assert_eq!(2, instance.invoke_named("rem_s", (20, 3)).unwrap());
    assert_eq!(0, instance.invoke_named("rem_s", (i32::MIN, -1)).unwrap());
    assert_eq!(0, instance.invoke_named("rem_s", (i32::MIN, 2)).unwrap());
}

/// A simple function to test signed remainder's RuntimeError when dividing by 0
#[test_log::test]
pub fn remainder_signed_panic_dividend_0() {
    let wat = String::from(REM_S_WAT).replace("{{TYPE}}", "i32");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let result = instance.invoke_named::<(i32, i32), i32>("rem_s", (222, 0));

    assert_eq!(result.unwrap_err(), RuntimeError::DivideBy0);
}

/// A simple function to test unsigned remainder
#[test_log::test]
pub fn i32_remainder_unsigned_simple() {
    let wat = String::from(REM_U_WAT).replace("{{TYPE}}", "i32");
    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(0, instance.invoke_named("rem_u", (i32::MIN, 2)).unwrap());
    assert_eq!(
        i32::MIN,
        instance.invoke_named("rem_u", (i32::MIN, -2)).unwrap()
    );
    assert_eq!(
        (i32::MIN + 2) * (-1),
        instance.invoke_named("rem_u", (-2, i32::MIN)).unwrap()
    );
    assert_eq!(2, instance.invoke_named("rem_u", (2, i32::MIN)).unwrap());
    assert_eq!(
        i32::MAX,
        instance
            .invoke_named("rem_u", (i32::MAX, i32::MIN))
            .unwrap()
    );

    assert_eq!(0, instance.invoke_named("rem_u", (20, 2)).unwrap());
    assert_eq!(
        999,
        instance.invoke_named("rem_u", (10_000, 9_001)).unwrap()
    );
    assert_eq!(2, instance.invoke_named("rem_u", (-20, 3)).unwrap());
    assert_eq!(-20, instance.invoke_named("rem_u", (-20, -3)).unwrap());
    assert_eq!(20, instance.invoke_named("rem_u", (20, -3)).unwrap());
    assert_eq!(2, instance.invoke_named("rem_u", (20, 3)).unwrap());
    assert_eq!(
        i32::MIN,
        instance.invoke_named("rem_u", (i32::MIN, -1)).unwrap()
    );
    assert_eq!(0, instance.invoke_named("rem_u", (i32::MIN, 2)).unwrap());
}

/// A simple function to test signed remainder's RuntimeError when dividing by 0
#[test_log::test]
pub fn i32_remainder_unsigned_panic_dividend_0() {
    let wat = String::from(REM_U_WAT).replace("{{TYPE}}", "i32");
    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let result = instance.invoke_named::<(i32, i32), i32>("rem_u", (222, 0));

    assert_eq!(result.unwrap_err(), RuntimeError::DivideBy0);
}
