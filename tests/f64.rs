#![allow(clippy::approx_constant)]
use core::f64;

use wasm::{validate, RuntimeInstance};

/// A simple function to test the f64.const implementation
#[test_log::test]
pub fn f64_const() {
    let wat = r#"
        (module
            (func (export "getF64Const") (result f64)
                f64.const 3.14159265359  ;; Pi
            )
        )
    "#;

    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");

    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        3.14159265359_f64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), ())
            .unwrap()
    );
}

/// A simple function to test the f64.eq implementation
#[test_log::test]
pub fn f64_eq() {
    let wat = r#"
        (module
            (func (export "f64_eq") (param $x f64) (param $y f64) (result i32)
                local.get $x
                local.get $y
                f64.eq
            )
        )
    "#;

    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");

    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1.1_f64, 1.1_f64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1.1_f64, 1.2_f64)
            )
            .unwrap()
    );
}

/// A simple function to test the f64.ne implementation
#[test_log::test]
pub fn f64_ne() {
    let wat = r#"
        (module
            (func (export "f64_ne") (param $x f64) (param $y f64) (result i32)
                local.get $x
                local.get $y
                f64.ne
            )
        )
    "#;

    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1.1_f64, 1.1_f64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1.1_f64, 1.2_f64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0.0_f64, -0.0_f64)
            )
            .unwrap()
    );
}

/// A simple function to test the f64.lt implementation
#[test_log::test]
pub fn f64_lt() {
    let wat = r#"
        (module
            (func (export "f64_lt") (param $x f64) (param $y f64) (result i32)
                local.get $x
                local.get $y
                f64.lt
            )
        )
    "#;

    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1.0_f64, 2.0_f64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (2.0_f64, 1.0_f64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1.0_f64, 1.0_f64)
            )
            .unwrap()
    );
}

/// A simple function to test the f64.gt implementation
#[test_log::test]
pub fn f64_gt() {
    let wat = r#"
        (module
            (func (export "f64_gt") (param $x f64) (param $y f64) (result i32)
                local.get $x
                local.get $y
                f64.gt
            )
        )
    "#;

    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1.0_f64, 2.0_f64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (2.0_f64, 1.0_f64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1.0_f64, 1.0_f64)
            )
            .unwrap()
    );
}

/// A simple function to test the f64.le implementation
#[test_log::test]
pub fn f64_le() {
    let wat = r#"
        (module
            (func (export "f64_le") (param $x f64) (param $y f64) (result i32)
                local.get $x
                local.get $y
                f64.le
            )
        )
    "#;

    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1.0_f64, 2.0_f64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (2.0_f64, 1.0_f64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1.0_f64, 1.0_f64)
            )
            .unwrap()
    );
}

/// A simple function to test the f64.ge implementation
#[test_log::test]
pub fn f64_ge() {
    let wat = r#"
        (module
            (func (export "f64_ge") (param $x f64) (param $y f64) (result i32)
                local.get $x
                local.get $y
                f64.ge
            )
        )
    "#;

    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1.0_f64, 2.0_f64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (2.0_f64, 1.0_f64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1.0_f64, 1.0_f64)
            )
            .unwrap()
    );
}

/// A simple function to test the f64.abs implementation
#[test_log::test]
pub fn f64_abs() {
    let wat = r#"
        (module
            (func (export "f64_abs") (param $x f64) (result f64)
                local.get $x
                f64.abs
            )
        )
    "#;

    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    {
        let result = instance
            .invoke::<f64, f64>(&instance.get_function_by_index(0, 0).unwrap(), -f64::NAN)
            .unwrap();
        assert!(result.is_nan());
        assert!(result.is_sign_positive());
    }
    {
        let result = instance
            .invoke::<f64, f64>(&instance.get_function_by_index(0, 0).unwrap(), f64::NAN)
            .unwrap();
        assert!(result.is_nan());
        assert!(result.is_sign_positive());
    }
    {
        let result = instance
            .invoke::<f64, f64>(
                &instance.get_function_by_index(0, 0).unwrap(),
                f64::NEG_INFINITY,
            )
            .unwrap();
        assert!(result.is_infinite());
        assert!(result.is_sign_positive());
    }
    {
        let result = instance
            .invoke::<f64, f64>(
                &instance.get_function_by_index(0, 0).unwrap(),
                f64::INFINITY,
            )
            .unwrap();
        assert!(result.is_infinite());
        assert!(result.is_sign_positive());
    }
    assert_eq!(
        1.5_f64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1.5_f64)
            .unwrap()
    );
    assert_eq!(
        1.5_f64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1.5_f64)
            .unwrap()
    );
    assert_eq!(
        0.0_f64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0.0_f64)
            .unwrap()
    );
    assert_eq!(
        0.0_f64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -0.0_f64)
            .unwrap()
    );
}

/// A simple function to test the f64.neg implementation
#[test_log::test]
pub fn f64_neg() {
    let wat = r#"
        (module
            (func (export "f64_neg") (param $x f64) (result f64)
                local.get $x
                f64.neg
            )
        )
    "#;

    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    {
        let result = instance
            .invoke::<f64, f64>(&instance.get_function_by_index(0, 0).unwrap(), -f64::NAN)
            .unwrap();
        assert!(result.is_nan());
        assert!(result.is_sign_positive());
    }
    {
        let result = instance
            .invoke::<f64, f64>(&instance.get_function_by_index(0, 0).unwrap(), f64::NAN)
            .unwrap();
        assert!(result.is_nan());
        assert!(result.is_sign_negative());
    }
    {
        let result = instance
            .invoke::<f64, f64>(
                &instance.get_function_by_index(0, 0).unwrap(),
                f64::NEG_INFINITY,
            )
            .unwrap();
        assert!(result.is_infinite());
        assert!(result.is_sign_positive());
    }
    {
        let result = instance
            .invoke::<f64, f64>(
                &instance.get_function_by_index(0, 0).unwrap(),
                f64::INFINITY,
            )
            .unwrap();
        assert!(result.is_infinite());
        assert!(result.is_sign_negative());
    }
    assert_eq!(
        -1.5_f64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1.5_f64)
            .unwrap()
    );
    assert_eq!(
        1.5_f64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1.5_f64)
            .unwrap()
    );
    assert_eq!(
        -0.0_f64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0.0_f64)
            .unwrap()
    );
    assert_eq!(
        0.0_f64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -0.0_f64)
            .unwrap()
    );
}

/// A simple function to test the f64.ceil implementation
#[test_log::test]
pub fn f64_ceil() {
    let wat = r#"
        (module
            (func (export "f64_ceil") (param $x f64) (result f64)
                local.get $x
                f64.ceil
            )
        )
    "#;

    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        2.0_f64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1.5_f64)
            .unwrap()
    );
    assert_eq!(
        -1.0_f64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1.5_f64)
            .unwrap()
    );
    assert_eq!(
        0.0_f64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -0.1_f64)
            .unwrap()
    );
}

/// A simple function to test the f64.floor implementation
#[test_log::test]
pub fn f64_floor() {
    let wat = r#"
        (module
            (func (export "f64_floor") (param $x f64) (result f64)
                local.get $x
                f64.floor
            )
        )
    "#;

    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        1.0_f64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1.5_f64)
            .unwrap()
    );
    assert_eq!(
        -2.0_f64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1.5_f64)
            .unwrap()
    );
    assert_eq!(
        -1.0_f64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -0.1_f64)
            .unwrap()
    );
}

/// A simple function to test the f64.trunc implementation
#[test_log::test]
pub fn f64_trunc() {
    let wat = r#"
        (module
            (func (export "f64_trunc") (param $x f64) (result f64)
                local.get $x
                f64.trunc
            )
        )
    "#;

    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        1.0_f64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1.5_f64)
            .unwrap()
    );
    assert_eq!(
        -1.0_f64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1.5_f64)
            .unwrap()
    );
    assert_eq!(
        0.0_f64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0.9_f64)
            .unwrap()
    );
}

/// A simple function to test the f64.nearest implementation
#[test_log::test]
pub fn f64_nearest() {
    let wat = r#"
        (module
            (func (export "f64_nearest") (param $x f64) (result f64)
                local.get $x
                f64.nearest
            )
        )
    "#;

    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        2.0_f64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1.5_f64)
            .unwrap()
    );
    assert_eq!(
        -2.0_f64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1.5_f64)
            .unwrap()
    );
    assert_eq!(
        1.0_f64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0.6_f64)
            .unwrap()
    );
    assert_eq!(
        0.0_f64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0.4_f64)
            .unwrap()
    );
}

/// A simple function to test the f64.sqrt implementation
#[test_log::test]
pub fn f64_sqrt() {
    let wat = r#"
        (module
            (func (export "f64_sqrt") (param $x f64) (result f64)
                local.get $x
                f64.sqrt
            )
        )
    "#;

    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        2.0_f64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 4.0_f64)
            .unwrap()
    );
    assert_eq!(
        1.4142135623730951_f64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 2.0_f64)
            .unwrap()
    );
    assert!(instance
        .invoke::<f64, f64>(&instance.get_function_by_index(0, 0).unwrap(), -f64::NAN)
        .unwrap()
        .is_nan());
}

/// A simple function to test the f64.add implementation
#[test_log::test]
pub fn f64_add() {
    let wat = r#"
        (module
            (func (export "f64_add") (param $x f64) (param $y f64) (result f64)
                local.get $x
                local.get $y
                f64.add
            )
        )
    "#;

    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        3.0_f64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1.5_f64, 1.5_f64)
            )
            .unwrap()
    );
    assert_eq!(
        -1.0_f64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1.0_f64, -2.0_f64)
            )
            .unwrap()
    );
    assert_eq!(
        0.0_f64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0.1_f64, -0.1_f64)
            )
            .unwrap()
    );
}

/// A simple function to test the f64.sub implementation
#[test_log::test]
pub fn f64_sub() {
    let wat = r#"
        (module
            (func (export "f64_sub") (param $x f64) (param $y f64) (result f64)
                local.get $x
                local.get $y
                f64.sub
            )
        )
    "#;

    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        0.0_f64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1.5_f64, 1.5_f64)
            )
            .unwrap()
    );
    assert_eq!(
        3.0_f64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1.0_f64, -2.0_f64)
            )
            .unwrap()
    );
    assert_eq!(
        0.2_f64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0.1_f64, -0.1_f64)
            )
            .unwrap()
    );
}

/// A simple function to test the f64.mul implementation
#[test_log::test]
pub fn f64_mul() {
    let wat = r#"
        (module
            (func (export "f64_mul") (param $x f64) (param $y f64) (result f64)
                local.get $x
                local.get $y
                f64.mul
            )
        )
    "#;

    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        6.0_f64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (2.0_f64, 3.0_f64)
            )
            .unwrap()
    );
    assert_eq!(
        -4.0_f64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (2.0_f64, -2.0_f64)
            )
            .unwrap()
    );
    assert_eq!(
        0.0_f64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0.0_f64, 5.0_f64)
            )
            .unwrap()
    );
}

/// A simple function to test the f64.div implementation
#[test_log::test]
pub fn f64_div() {
    let wat = r#"
        (module
            (func (export "f64_div") (param $x f64) (param $y f64) (result f64)
                local.get $x
                local.get $y
                f64.div
            )
        )
    "#;

    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        2.0_f64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (6.0_f64, 3.0_f64)
            )
            .unwrap()
    );
    assert_eq!(
        -1.0_f64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (2.0_f64, -2.0_f64)
            )
            .unwrap()
    );
    assert!(instance
        .invoke::<(f64, f64), f64>(
            &instance.get_function_by_index(0, 0).unwrap(),
            (1.0_f64, 0.0_f64)
        )
        .unwrap()
        .is_infinite());
    assert!(instance
        .invoke::<(f64, f64), f64>(
            &instance.get_function_by_index(0, 0).unwrap(),
            (0.0_f64, 0.0_f64)
        )
        .unwrap()
        .is_nan());
}

/// A simple function to test the f64.min implementation
#[test_log::test]
pub fn f64_min() {
    let wat = r#"
        (module
            (func (export "f64_min") (param $x f64) (param $y f64) (result f64)
                local.get $x
                local.get $y
                f64.min
            )
        )
    "#;

    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    {
        let result = instance
            .invoke::<(f64, f64), f64>(
                &instance.get_function_by_index(0, 0).unwrap(),
                (f64::NAN, -f64::NAN),
            )
            .unwrap();
        assert!(result.is_nan());
    }
    {
        let result = instance
            .invoke::<(f64, f64), f64>(
                &instance.get_function_by_index(0, 0).unwrap(),
                (f64::NAN, f64::NAN),
            )
            .unwrap();
        assert!(result.is_nan());
        assert!(result.is_sign_positive());
    }
    {
        let result = instance
            .invoke::<(f64, f64), f64>(
                &instance.get_function_by_index(0, 0).unwrap(),
                (f64::INFINITY, f64::NEG_INFINITY),
            )
            .unwrap();
        assert!(result.is_infinite());
        assert!(result.is_sign_negative());
    }
    assert_eq!(
        42_f64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (f64::INFINITY, 42_f64)
            )
            .unwrap()
    );
    assert_eq!(
        -0_f64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-0_f64, 0_f64)
            )
            .unwrap()
    );
    assert_eq!(
        1.0_f64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1.0_f64, 2.0_f64)
            )
            .unwrap()
    );
    assert_eq!(
        -2.0_f64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1.0_f64, -2.0_f64)
            )
            .unwrap()
    );
    assert_eq!(
        -0.0_f64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0.0_f64, -0.0_f64)
            )
            .unwrap()
    );
    assert!(instance
        .invoke::<(f64, f64), f64>(
            &instance.get_function_by_index(0, 0).unwrap(),
            (f64::NAN, 1.0_f64)
        )
        .unwrap()
        .is_nan());
}

/// A simple function to test the f64.max implementation
#[test_log::test]
pub fn f64_max() {
    let wat = r#"
        (module
            (func (export "f64_max") (param $x f64) (param $y f64) (result f64)
                local.get $x
                local.get $y
                f64.max
            )
        )
    "#;

    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    {
        let result = instance
            .invoke::<(f64, f64), f64>(
                &instance.get_function_by_index(0, 0).unwrap(),
                (f64::NAN, -f64::NAN),
            )
            .unwrap();
        assert!(result.is_nan());
    }
    {
        let result = instance
            .invoke::<(f64, f64), f64>(
                &instance.get_function_by_index(0, 0).unwrap(),
                (f64::NAN, f64::NAN),
            )
            .unwrap();
        assert!(result.is_nan());
        assert!(result.is_sign_positive());
    }
    {
        let result = instance
            .invoke::<(f64, f64), f64>(
                &instance.get_function_by_index(0, 0).unwrap(),
                (f64::INFINITY, f64::NEG_INFINITY),
            )
            .unwrap();
        assert!(result.is_infinite());
        assert!(result.is_sign_positive());
    }
    assert_eq!(
        42_f64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (f64::NEG_INFINITY, 42_f64)
            )
            .unwrap()
    );
    assert_eq!(
        0_f64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-0_f64, 0_f64)
            )
            .unwrap()
    );

    assert_eq!(
        2.0_f64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1.0_f64, 2.0_f64)
            )
            .unwrap()
    );
    assert_eq!(
        -1.0_f64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1.0_f64, -2.0_f64)
            )
            .unwrap()
    );
    assert_eq!(
        0.0_f64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0.0_f64, -0.0_f64)
            )
            .unwrap()
    );
    assert!(instance
        .invoke::<(f64, f64), f64>(
            &instance.get_function_by_index(0, 0).unwrap(),
            (f64::NAN, 1.0_f64)
        )
        .unwrap()
        .is_nan());
}

/// A simple function to test the f64.copysign implementation
#[test_log::test]
pub fn f64_copysign() {
    let wat = r#"
        (module
            (func (export "f64_copysign") (param $x f64) (param $y f64) (result f64)
                local.get $x
                local.get $y
                f64.copysign
            )
        )
    "#;

    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        1.5_f64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1.5_f64, 2.0_f64)
            )
            .unwrap()
    );
    assert_eq!(
        -1.5_f64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1.5_f64, -2.0_f64)
            )
            .unwrap()
    );
    assert_eq!(
        -1.5_f64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1.5_f64, -0.0_f64)
            )
            .unwrap()
    );
    assert_eq!(
        1.5_f64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1.5_f64, 0.0_f64)
            )
            .unwrap()
    );
}
