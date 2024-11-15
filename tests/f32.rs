use core::f32;

use wasm::{validate, RuntimeInstance};

/// A simple function to test the f32.const implementation
#[test_log::test]
pub fn f32_const() {
    let wat = r#"
        (module
            (func (export "getF32Const") (result f32)
                f32.const 3.14159274  ;; Pi
            )
        )
    "#;

    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");

    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        3.141_592_7_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), ())
            .unwrap()
    );
}

const WAT_2_ARGS_RETURN_I32: &str = r#"
    (module
        (func (export "f32_{{0}}") (param $x f32) (param $y f32) (result i32)
            local.get $x
            local.get $y
            f32.{{0}}
        )
    )
"#;

/// A simple function to test the f32.eq implementation
#[test_log::test]
pub fn f32_eq() {
    let wat = String::from(WAT_2_ARGS_RETURN_I32).replace("{{0}}", "eq");

    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");

    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1.1_f32, 1.1_f32)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1.1_f32, 1.2_f32)
            )
            .unwrap()
    );
}

/// A simple function to test the f32.ne implementation
#[test_log::test]
pub fn f32_ne() {
    let wat = String::from(WAT_2_ARGS_RETURN_I32).replace("{{0}}", "ne");

    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1.1_f32, 1.1_f32)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1.1_f32, 1.2_f32)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0.0_f32, -0.0_f32)
            )
            .unwrap()
    );
}

/// A simple function to test the f32.lt implementation
#[test_log::test]
pub fn f32_lt() {
    let wat = String::from(WAT_2_ARGS_RETURN_I32).replace("{{0}}", "lt");

    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1.0_f32, 2.0_f32)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (2.0_f32, 1.0_f32)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1.0_f32, 1.0_f32)
            )
            .unwrap()
    );
}

/// A simple function to test the f32.gt implementation
#[test_log::test]
pub fn f32_gt() {
    let wat = String::from(WAT_2_ARGS_RETURN_I32).replace("{{0}}", "gt");

    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1.0_f32, 2.0_f32)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (2.0_f32, 1.0_f32)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1.0_f32, 1.0_f32)
            )
            .unwrap()
    );
}

/// A simple function to test the f32.le implementation
#[test_log::test]
pub fn f32_le() {
    let wat = String::from(WAT_2_ARGS_RETURN_I32).replace("{{0}}", "le");

    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1.0_f32, 2.0_f32)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (2.0_f32, 1.0_f32)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1.0_f32, 1.0_f32)
            )
            .unwrap()
    );
}

/// A simple function to test the f32.ge implementation
#[test_log::test]
pub fn f32_ge() {
    let wat = String::from(WAT_2_ARGS_RETURN_I32).replace("{{0}}", "ge");

    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1.0_f32, 2.0_f32)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (2.0_f32, 1.0_f32)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1.0_f32, 1.0_f32)
            )
            .unwrap()
    );
}

const WAT_1_ARG_RETURN_F32: &str = r#"
    (module
      (func (export "f32_{{0}}") (param $x f32) (result f32)
          local.get $x
          f32.{{0}})
    )
"#;

/// A simple function to test the f32.abs implementation
#[test_log::test]
pub fn f32_abs() {
    let wat = String::from(WAT_1_ARG_RETURN_F32).replace("{{0}}", "abs");

    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    {
        let result = instance
            .invoke::<f32, f32>(&instance.get_function_by_index(0, 0).unwrap(), -f32::NAN)
            .unwrap();
        assert!(result.is_nan());
        assert!(result.is_sign_positive());
    }
    {
        let result = instance
            .invoke::<f32, f32>(&instance.get_function_by_index(0, 0).unwrap(), f32::NAN)
            .unwrap();
        assert!(result.is_nan());
        assert!(result.is_sign_positive());
    }
    {
        let result = instance
            .invoke::<f32, f32>(
                &instance.get_function_by_index(0, 0).unwrap(),
                f32::NEG_INFINITY,
            )
            .unwrap();
        assert!(result.is_infinite());
        assert!(result.is_sign_positive());
    }
    {
        let result = instance
            .invoke::<f32, f32>(
                &instance.get_function_by_index(0, 0).unwrap(),
                f32::INFINITY,
            )
            .unwrap();
        assert!(result.is_infinite());
        assert!(result.is_sign_positive());
    }
    assert_eq!(
        1.5_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1.5_f32)
            .unwrap()
    );
    assert_eq!(
        1.5_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1.5_f32)
            .unwrap()
    );
    assert_eq!(
        0.0_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0.0_f32)
            .unwrap()
    );
    assert_eq!(
        0.0_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -0.0_f32)
            .unwrap()
    );
}

/// A simple function to test the f32.neg implementation
#[test_log::test]
pub fn f32_neg() {
    let wat = String::from(WAT_1_ARG_RETURN_F32).replace("{{0}}", "neg");

    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    {
        let result = instance
            .invoke::<f32, f32>(&instance.get_function_by_index(0, 0).unwrap(), -f32::NAN)
            .unwrap();
        assert!(result.is_nan());
        assert!(result.is_sign_positive());
    }
    {
        let result = instance
            .invoke::<f32, f32>(&instance.get_function_by_index(0, 0).unwrap(), f32::NAN)
            .unwrap();
        assert!(result.is_nan());
        assert!(result.is_sign_negative());
    }
    {
        let result = instance
            .invoke::<f32, f32>(
                &instance.get_function_by_index(0, 0).unwrap(),
                f32::NEG_INFINITY,
            )
            .unwrap();
        assert!(result.is_infinite());
        assert!(result.is_sign_positive());
    }
    {
        let result = instance
            .invoke::<f32, f32>(
                &instance.get_function_by_index(0, 0).unwrap(),
                f32::INFINITY,
            )
            .unwrap();
        assert!(result.is_infinite());
        assert!(result.is_sign_negative());
    }
    assert_eq!(
        -1.5_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1.5_f32)
            .unwrap()
    );
    assert_eq!(
        1.5_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1.5_f32)
            .unwrap()
    );
    assert_eq!(
        -0.0_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0.0_f32)
            .unwrap()
    );
    assert_eq!(
        0.0_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -0.0_f32)
            .unwrap()
    );
}

/// A simple function to test the f32.ceil implementation
#[test_log::test]
pub fn f32_ceil() {
    let wat = String::from(WAT_1_ARG_RETURN_F32).replace("{{0}}", "ceil");

    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        2.0_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1.5_f32)
            .unwrap()
    );
    assert_eq!(
        -1.0_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1.5_f32)
            .unwrap()
    );
    assert_eq!(
        0.0_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -0.1_f32)
            .unwrap()
    );
}

/// A simple function to test the f32.floor implementation
#[test_log::test]
pub fn f32_floor() {
    let wat = String::from(WAT_1_ARG_RETURN_F32).replace("{{0}}", "floor");

    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        1.0_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1.5_f32)
            .unwrap()
    );
    assert_eq!(
        -2.0_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1.5_f32)
            .unwrap()
    );
    assert_eq!(
        -1.0_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -0.1_f32)
            .unwrap()
    );
}

/// A simple function to test the f32.trunc implementation
#[test_log::test]
pub fn f32_trunc() {
    let wat = String::from(WAT_1_ARG_RETURN_F32).replace("{{0}}", "trunc");

    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        1.0_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1.5_f32)
            .unwrap()
    );
    assert_eq!(
        -1.0_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1.5_f32)
            .unwrap()
    );
    assert_eq!(
        0.0_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0.9_f32)
            .unwrap()
    );
}

/// A simple function to test the f32.nearest implementation
#[test_log::test]
pub fn f32_nearest() {
    let wat = String::from(WAT_1_ARG_RETURN_F32).replace("{{0}}", "nearest");

    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        2.0_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1.5_f32)
            .unwrap()
    );
    assert_eq!(
        -2.0_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1.5_f32)
            .unwrap()
    );
    assert_eq!(
        1.0_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0.6_f32)
            .unwrap()
    );
    assert_eq!(
        0.0_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0.4_f32)
            .unwrap()
    );
}

/// A simple function to test the f32.sqrt implementation
#[test_log::test]
pub fn f32_sqrt() {
    let wat = String::from(WAT_1_ARG_RETURN_F32).replace("{{0}}", "sqrt");

    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        2.0_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 4.0_f32)
            .unwrap()
    );
    assert_eq!(
        1.4142135_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 2.0_f32)
            .unwrap()
    );
    assert!(instance
        .invoke::<f32, f32>(&instance.get_function_by_index(0, 0).unwrap(), -f32::NAN)
        .unwrap()
        .is_nan());
}

const WAT_2_ARGS_RETURN_F32: &str = r#"
    (module
      (func (export "f32_{{0}}") (param $x f32) (param $y f32) (result f32)
          local.get $x
          local.get $y
          f32.{{0}})
    )
"#;

/// A simple function to test the f32.add implementation
#[test_log::test]
pub fn f32_add() {
    let wat = String::from(WAT_2_ARGS_RETURN_F32).replace("{{0}}", "add");

    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        3.0_f32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1.5_f32, 1.5_f32)
            )
            .unwrap()
    );
    assert_eq!(
        -1.0_f32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1.0_f32, -2.0_f32)
            )
            .unwrap()
    );
    assert_eq!(
        0.0_f32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0.1_f32, -0.1_f32)
            )
            .unwrap()
    );
}

/// A simple function to test the f32.sub implementation
#[test_log::test]
pub fn f32_sub() {
    let wat = String::from(WAT_2_ARGS_RETURN_F32).replace("{{0}}", "sub");

    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        0.0_f32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1.5_f32, 1.5_f32)
            )
            .unwrap()
    );
    assert_eq!(
        3.0_f32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1.0_f32, -2.0_f32)
            )
            .unwrap()
    );
    assert_eq!(
        0.2_f32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0.1_f32, -0.1_f32)
            )
            .unwrap()
    );
}

/// A simple function to test the f32.mul implementation
#[test_log::test]
pub fn f32_mul() {
    let wat = String::from(WAT_2_ARGS_RETURN_F32).replace("{{0}}", "mul");

    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        6.0_f32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (2.0_f32, 3.0_f32)
            )
            .unwrap()
    );
    assert_eq!(
        -4.0_f32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (2.0_f32, -2.0_f32)
            )
            .unwrap()
    );
    assert_eq!(
        0.0_f32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0.0_f32, 5.0_f32)
            )
            .unwrap()
    );
}

/// A simple function to test the f32.div implementation
#[test_log::test]
pub fn f32_div() {
    let wat = String::from(WAT_2_ARGS_RETURN_F32).replace("{{0}}", "div");

    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        2.0_f32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (6.0_f32, 3.0_f32)
            )
            .unwrap()
    );
    assert_eq!(
        -1.0_f32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (2.0_f32, -2.0_f32)
            )
            .unwrap()
    );
    assert!(instance
        .invoke::<(f32, f32), f32>(
            &instance.get_function_by_index(0, 0).unwrap(),
            (1.0_f32, 0.0_f32)
        )
        .unwrap()
        .is_infinite());
    assert!(instance
        .invoke::<(f32, f32), f32>(
            &instance.get_function_by_index(0, 0).unwrap(),
            (0.0_f32, 0.0_f32)
        )
        .unwrap()
        .is_nan());
}

/// A simple function to test the f32.min implementation
#[test_log::test]
pub fn f32_min() {
    let wat = String::from(WAT_2_ARGS_RETURN_F32).replace("{{0}}", "min");

    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    {
        let result = instance
            .invoke::<(f32, f32), f32>(
                &instance.get_function_by_index(0, 0).unwrap(),
                (f32::NAN, -f32::NAN),
            )
            .unwrap();
        assert!(result.is_nan());
    }
    {
        let result = instance
            .invoke::<(f32, f32), f32>(
                &instance.get_function_by_index(0, 0).unwrap(),
                (f32::NAN, f32::NAN),
            )
            .unwrap();
        assert!(result.is_nan());
        assert!(result.is_sign_positive());
    }
    {
        let result = instance
            .invoke::<(f32, f32), f32>(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-f32::NAN, -f32::NAN),
            )
            .unwrap();
        assert!(result.is_nan());
        assert!(result.is_sign_negative());
    }
    {
        let result = instance
            .invoke::<(f32, f32), f32>(
                &instance.get_function_by_index(0, 0).unwrap(),
                (f32::INFINITY, f32::NEG_INFINITY),
            )
            .unwrap();
        assert!(result.is_infinite());
        assert!(result.is_sign_negative());
    }
    assert_eq!(
        42_f32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (f32::INFINITY, 42_f32)
            )
            .unwrap()
    );
    assert_eq!(
        -0_f32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-0_f32, 0_f32)
            )
            .unwrap()
    );
    assert_eq!(
        1.0_f32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1.0_f32, 2.0_f32)
            )
            .unwrap()
    );
    assert_eq!(
        -2.0_f32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1.0_f32, -2.0_f32)
            )
            .unwrap()
    );
    assert_eq!(
        -0.0_f32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0.0_f32, -0.0_f32)
            )
            .unwrap()
    );
    assert!(instance
        .invoke::<(f32, f32), f32>(
            &instance.get_function_by_index(0, 0).unwrap(),
            (f32::NAN, 1.0_f32)
        )
        .unwrap()
        .is_nan());
}

/// A simple function to test the f32.max implementation
#[test_log::test]
pub fn f32_max() {
    let wat = String::from(WAT_2_ARGS_RETURN_F32).replace("{{0}}", "max");

    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    {
        let result = instance
            .invoke::<(f32, f32), f32>(
                &instance.get_function_by_index(0, 0).unwrap(),
                (f32::NAN, -f32::NAN),
            )
            .unwrap();
        assert!(result.is_nan());
    }
    {
        let result = instance
            .invoke::<(f32, f32), f32>(
                &instance.get_function_by_index(0, 0).unwrap(),
                (f32::NAN, f32::NAN),
            )
            .unwrap();
        assert!(result.is_nan());
        assert!(result.is_sign_positive());
    }
    {
        let result = instance
            .invoke::<(f32, f32), f32>(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-f32::NAN, -f32::NAN),
            )
            .unwrap();
        assert!(result.is_nan());
        assert!(result.is_sign_negative());
    }
    {
        let result = instance
            .invoke::<(f32, f32), f32>(
                &instance.get_function_by_index(0, 0).unwrap(),
                (f32::INFINITY, f32::NEG_INFINITY),
            )
            .unwrap();
        assert!(result.is_infinite());
        assert!(result.is_sign_positive());
    }
    assert_eq!(
        42_f32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (f32::NEG_INFINITY, 42_f32)
            )
            .unwrap()
    );
    assert_eq!(
        0_f32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-0_f32, 0_f32)
            )
            .unwrap()
    );

    assert_eq!(
        2.0_f32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1.0_f32, 2.0_f32)
            )
            .unwrap()
    );
    assert_eq!(
        -1.0_f32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1.0_f32, -2.0_f32)
            )
            .unwrap()
    );
    assert_eq!(
        0.0_f32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0.0_f32, -0.0_f32)
            )
            .unwrap()
    );
    assert!(instance
        .invoke::<(f32, f32), f32>(
            &instance.get_function_by_index(0, 0).unwrap(),
            (f32::NAN, 1.0_f32)
        )
        .unwrap()
        .is_nan());
}

/// A simple function to test the f32.copysign implementation
#[test_log::test]
pub fn f32_copysign() {
    let wat = String::from(WAT_2_ARGS_RETURN_F32).replace("{{0}}", "copysign");

    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        1.5_f32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1.5_f32, 2.0_f32)
            )
            .unwrap()
    );
    assert_eq!(
        -1.5_f32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1.5_f32, -2.0_f32)
            )
            .unwrap()
    );
    assert_eq!(
        -1.5_f32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1.5_f32, -0.0_f32)
            )
            .unwrap()
    );
    assert_eq!(
        1.5_f32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1.5_f32, 0.0_f32)
            )
            .unwrap()
    );
}

#[test_log::test]
pub fn f32_convert_i32_s() {
    let wat = r#"
        (module
            (func (export "convert_i32_s") (param $x i32) (result f32)
                local.get $x
                f32.convert_i32_s
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let i32_s_val = -42_i32;
    let f32_result = instance
        .invoke::<i32, f32>(&instance.get_function_by_index(0, 0).unwrap(), i32_s_val)
        .unwrap();
    assert_eq!(f32_result, -42.0_f32);
}

#[test_log::test]
pub fn f32_convert_i32_u() {
    let wat = r#"
        (module
            (func (export "convert_i32_u") (param $x i32) (result f32)
                local.get $x
                f32.convert_i32_u
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let test_cases: Vec<(i32, f32)> = vec![
        (-2147483648, 2147483648.0),
        (0x12345678, 305419900.0),
        (0xffffffffu32 as i32, 4294967296.0),
        (0x80000080u32 as i32, 2147483600.0),
        (0x80000081u32 as i32, 2147484000.0),
        (0x80000082u32 as i32, 2147484000.0),
        (0xfffffe80u32 as i32, 4294966800.0),
    ];

    for (input, expected) in test_cases {
        let result = instance
            .invoke::<i32, f32>(&instance.get_function_by_index(0, 0).unwrap(), input)
            .unwrap();
        assert_eq!(
            result, expected,
            "Failed for input: {} (0x{:X})",
            input, input as u32
        );
    }

    // Test for precision loss
    let large_value = 0xFFFFFFFF_u32 as i32; // Maximum u32 value
    let result = instance
        .invoke::<i32, f32>(&instance.get_function_by_index(0, 0).unwrap(), large_value)
        .unwrap();
    assert!(
        result > 4294967040.0 && result <= 4294967296.0,
        "Large value conversion imprecise: got {}",
        result
    );
}

#[test_log::test]
pub fn f32_convert_i64_s() {
    let wat = r#"
        (module
            (func (export "convert_i64_s") (param $x i64) (result f32)
                local.get $x
                f32.convert_i64_s
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let i64_s_val = i64::MIN; // Minimum i64 value
    let f32_result: f32 = instance
        .invoke::<i64, f32>(&instance.get_function_by_index(0, 0).unwrap(), i64_s_val)
        .unwrap();
    assert_eq!(f32_result, i64::MIN as f32);

    assert_eq!(
        9223371500000000000.0,
        instance
            .invoke::<i64, f32>(
                &instance.get_function_by_index(0, 0).unwrap(),
                0x7fffff4000000001_i64
            )
            .unwrap()
    );
    assert_eq!(
        -9223371500000000000.0,
        instance
            .invoke::<i64, f32>(
                &instance.get_function_by_index(0, 0).unwrap(),
                0x8000004000000001_u64 as i64
            )
            .unwrap()
    );
}

#[test_log::test]
pub fn f32_convert_i64_u() {
    let wat = r#"
        (module
            (func (export "convert_i64_u") (param $x i64) (result f32)
                local.get $x
                f32.convert_i64_u
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        9223373000000000000.0,
        instance
            .invoke::<i64, f32>(
                &instance.get_function_by_index(0, 0).unwrap(),
                0x8000008000000001u64 as i64
            )
            .unwrap()
    );
    assert_eq!(
        18446743000000000000.0,
        instance
            .invoke::<i64, f32>(
                &instance.get_function_by_index(0, 0).unwrap(),
                0xfffffe8000000001u64 as i64
            )
            .unwrap()
    );
}

#[test_log::test]
pub fn f32_reinterpret_i32() {
    let wat = r#"
        (module
            (func (export "reinterpret_i32") (param $x i32) (result f32)
                local.get $x
                f32.reinterpret_i32
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let test_cases = vec![
        (0x00000000, 0.0), // Positive zero
        // (0x80000000, -0.0),  // Negative zero
        (0x3f800000, 1.0), // One
        // (0xbf800000, -1.0),  // Negative one
        (0x7f800000, f32::INFINITY), // Positive infinity
        // (0xff800000, f32::NEG_INFINITY),  // Negative infinity
        (0x7fc00000, f32::NAN), // NaN
    ];

    for (input, expected) in test_cases {
        let result = instance
            .invoke::<i32, f32>(&instance.get_function_by_index(0, 0).unwrap(), input)
            .unwrap();
        if expected.is_nan() {
            assert!(result.is_nan(), "Failed for input: {:x}", input);
        } else {
            assert_eq!(result, expected, "Failed for input: {:x}", input);
        }
    }
}
