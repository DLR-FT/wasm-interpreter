/// A simple function to test signed division
#[test_log::test]
pub fn division_signed_simple() {
    use wasm::{validate, RuntimeInstance};

    let wat = r#"
    (module
        (func (export "signed_division") (param $divisor i32) (param $dividend i32) (result i32)
            local.get $divisor
            local.get $dividend
            i32.div_s)
    )
    "#;

    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");

    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(10, instance.invoke_func(0, (20, 2)).unwrap());
    assert_eq!(9_001, instance.invoke_func(0, (81_018_001, 9_001)).unwrap());
    assert_eq!(-10, instance.invoke_func(0, (20, -2)).unwrap());
    assert_eq!(10, instance.invoke_func(0, (-20, -2)).unwrap());
    assert_eq!(-10, instance.invoke_func(0, (-20, 2)).unwrap());
}

/// A simple function to test signed division's RuntimeError when dividing by 0
#[test_log::test]
pub fn division_signed_panic_dividend_0() {
    use wasm::{validate, RuntimeInstance};

    let wat = r#"
  (module
      (func (export "signed_division") (param $divisor i32) (param $dividend i32) (result i32)
          local.get $divisor
          local.get $dividend
          i32.div_s)
  )
  "#;

    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");

    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let result = instance.invoke_func::<(i32, i32), i32>(0, (222, 0));

    assert_eq!(
        result.unwrap_err(),
        wasm::Error::RuntimeError(wasm::RuntimeError::DivideBy0)
    );
}

/// A simple function to test signed division's RuntimeError when we are dividing the i32 minimum by -1 (which gives an unrepresentable result - overflow)
#[test_log::test]
pub fn division_signed_panic_result_unrepresentable() {
    use wasm::{validate, RuntimeInstance};

    let wat = r#"
  (module
      (func (export "signed_division") (param $divisor i32) (param $dividend i32) (result i32)
          local.get $divisor
          local.get $dividend
          i32.div_s)
  )
  "#;

    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");

    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let result = instance.invoke_func::<(i32, i32), i32>(0, (i32::MIN, -1));

    assert_eq!(
        result.unwrap_err(),
        wasm::Error::RuntimeError(wasm::RuntimeError::UnrepresentableResult)
    );
}
