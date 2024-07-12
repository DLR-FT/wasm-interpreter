const REM_S_WAT: &'static str = r#"
    (module
        (func (export "rem_s") (param $divisor i32) (param $dividend i32) (result i32)
            local.get $divisor
            local.get $dividend
            i32.rem_s)
    )
"#;

const REM_U_WAT: &'static str = r#"
    (module
        (func (export "rem_u") (param $divisor i32) (param $dividend i32) (result i32)
            local.get $divisor
            local.get $dividend
            i32.rem_u)
    )
"#;

/// A simple function to test signed remainder
#[test_log::test]
pub fn remainder_signed_simple() {
    use wasm::{validate, RuntimeInstance};

    let wat = REM_S_WAT;
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
    use wasm::{validate, RuntimeInstance};

    let wat = REM_S_WAT;
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let result = instance.invoke_named::<(i32, i32), i32>("rem_s", (222, 0));

    assert_eq!(
        result.unwrap_err(),
        wasm::Error::RuntimeError(wasm::RuntimeError::DivideBy0)
    );
}

/// A simple function to test unsigned remainder
#[test_log::test]
pub fn remainder_unsigned_simple() {
    use wasm::{validate, RuntimeInstance};

    let wat = REM_U_WAT;
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
pub fn remainder_unsigned_panic_dividend_0() {
    use wasm::{validate, RuntimeInstance};

    let wat = REM_U_WAT;
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let result = instance.invoke_named::<(i32, i32), i32>("rem_u", (222, 0));

    assert_eq!(
        result.unwrap_err(),
        wasm::Error::RuntimeError(wasm::RuntimeError::DivideBy0)
    );
}
