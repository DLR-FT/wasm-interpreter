#![allow(clippy::approx_constant)]
use core::f64;

use wasm::{validate, Store};

/// A simple function to test the f64.const implementation
#[test_log::test]
pub fn f64_const() {
    let wat = r#"
        (module
            (func (export "get_f64_const") (result f64)
                f64.const 3.14159265359  ;; Pi
            )
        )
    "#;

    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");

    let mut store = Store::new(());
    let module = store
        .module_instantiate(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;

    let function = store
        .instance_export(module, "get_f64_const")
        .unwrap()
        .as_func()
        .unwrap();

    assert_eq!(
        3.14159265359_f64,
        store.invoke_typed_without_fuel(function, ()).unwrap()
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

    let mut store = Store::new(());
    let module = store
        .module_instantiate(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;

    let function = store
        .instance_export(module, "f64_eq")
        .unwrap()
        .as_func()
        .unwrap();

    assert_eq!(
        1,
        store
            .invoke_typed_without_fuel(function, (1.1_f64, 1.1_f64))
            .unwrap()
    );
    assert_eq!(
        0,
        store
            .invoke_typed_without_fuel(function, (1.1_f64, 1.2_f64))
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
    let mut store = Store::new(());
    let module = store
        .module_instantiate(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;

    let function = store
        .instance_export(module, "f64_ne")
        .unwrap()
        .as_func()
        .unwrap();

    assert_eq!(
        0,
        store
            .invoke_typed_without_fuel(function, (1.1_f64, 1.1_f64))
            .unwrap()
    );
    assert_eq!(
        1,
        store
            .invoke_typed_without_fuel(function, (1.1_f64, 1.2_f64))
            .unwrap()
    );
    assert_eq!(
        0,
        store
            .invoke_typed_without_fuel(function, (0.0_f64, -0.0_f64))
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
    let mut store = Store::new(());
    let module = store
        .module_instantiate(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;

    let function = store
        .instance_export(module, "f64_lt")
        .unwrap()
        .as_func()
        .unwrap();

    assert_eq!(
        1,
        store
            .invoke_typed_without_fuel(function, (1.0_f64, 2.0_f64))
            .unwrap()
    );
    assert_eq!(
        0,
        store
            .invoke_typed_without_fuel(function, (2.0_f64, 1.0_f64))
            .unwrap()
    );
    assert_eq!(
        0,
        store
            .invoke_typed_without_fuel(function, (1.0_f64, 1.0_f64))
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
    let mut store = Store::new(());
    let module = store
        .module_instantiate(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;

    let function = store
        .instance_export(module, "f64_gt")
        .unwrap()
        .as_func()
        .unwrap();

    assert_eq!(
        0,
        store
            .invoke_typed_without_fuel(function, (1.0_f64, 2.0_f64))
            .unwrap()
    );
    assert_eq!(
        1,
        store
            .invoke_typed_without_fuel(function, (2.0_f64, 1.0_f64))
            .unwrap()
    );
    assert_eq!(
        0,
        store
            .invoke_typed_without_fuel(function, (1.0_f64, 1.0_f64))
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
    let mut store = Store::new(());
    let module = store
        .module_instantiate(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;

    let function = store
        .instance_export(module, "f64_le")
        .unwrap()
        .as_func()
        .unwrap();

    assert_eq!(
        1,
        store
            .invoke_typed_without_fuel(function, (1.0_f64, 2.0_f64))
            .unwrap()
    );
    assert_eq!(
        0,
        store
            .invoke_typed_without_fuel(function, (2.0_f64, 1.0_f64))
            .unwrap()
    );
    assert_eq!(
        1,
        store
            .invoke_typed_without_fuel(function, (1.0_f64, 1.0_f64))
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
    let mut store = Store::new(());
    let module = store
        .module_instantiate(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;

    let function = store
        .instance_export(module, "f64_ge")
        .unwrap()
        .as_func()
        .unwrap();

    assert_eq!(
        0,
        store
            .invoke_typed_without_fuel(function, (1.0_f64, 2.0_f64))
            .unwrap()
    );
    assert_eq!(
        1,
        store
            .invoke_typed_without_fuel(function, (2.0_f64, 1.0_f64))
            .unwrap()
    );
    assert_eq!(
        1,
        store
            .invoke_typed_without_fuel(function, (1.0_f64, 1.0_f64))
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
    let mut store = Store::new(());
    let module = store
        .module_instantiate(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;

    let function = store
        .instance_export(module, "f64_abs")
        .unwrap()
        .as_func()
        .unwrap();

    {
        let result = store
            .invoke_typed_without_fuel::<f64, f64>(function, -f64::NAN)
            .unwrap();
        assert!(result.is_nan());
        assert!(result.is_sign_positive());
    }
    {
        let result = store
            .invoke_typed_without_fuel::<f64, f64>(function, f64::NAN)
            .unwrap();
        assert!(result.is_nan());
        assert!(result.is_sign_positive());
    }
    {
        let result = store
            .invoke_typed_without_fuel::<f64, f64>(function, f64::NEG_INFINITY)
            .unwrap();
        assert!(result.is_infinite());
        assert!(result.is_sign_positive());
    }
    {
        let result = store
            .invoke_typed_without_fuel::<f64, f64>(function, f64::INFINITY)
            .unwrap();
        assert!(result.is_infinite());
        assert!(result.is_sign_positive());
    }
    assert_eq!(
        1.5_f64,
        store.invoke_typed_without_fuel(function, 1.5_f64).unwrap()
    );
    assert_eq!(
        1.5_f64,
        store.invoke_typed_without_fuel(function, -1.5_f64).unwrap()
    );
    assert_eq!(
        0.0_f64,
        store.invoke_typed_without_fuel(function, 0.0_f64).unwrap()
    );
    assert_eq!(
        0.0_f64,
        store.invoke_typed_without_fuel(function, -0.0_f64).unwrap()
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
    let mut store = Store::new(());
    let module = store
        .module_instantiate(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;

    let function = store
        .instance_export(module, "f64_neg")
        .unwrap()
        .as_func()
        .unwrap();

    {
        let result = store
            .invoke_typed_without_fuel::<f64, f64>(function, -f64::NAN)
            .unwrap();
        assert!(result.is_nan());
        assert!(result.is_sign_positive());
    }
    {
        let result = store
            .invoke_typed_without_fuel::<f64, f64>(function, f64::NAN)
            .unwrap();
        assert!(result.is_nan());
        assert!(result.is_sign_negative());
    }
    {
        let result = store
            .invoke_typed_without_fuel::<f64, f64>(function, f64::NEG_INFINITY)
            .unwrap();
        assert!(result.is_infinite());
        assert!(result.is_sign_positive());
    }
    {
        let result = store
            .invoke_typed_without_fuel::<f64, f64>(function, f64::INFINITY)
            .unwrap();
        assert!(result.is_infinite());
        assert!(result.is_sign_negative());
    }
    assert_eq!(
        -1.5_f64,
        store.invoke_typed_without_fuel(function, 1.5_f64).unwrap()
    );
    assert_eq!(
        1.5_f64,
        store.invoke_typed_without_fuel(function, -1.5_f64).unwrap()
    );
    assert_eq!(
        -0.0_f64,
        store.invoke_typed_without_fuel(function, 0.0_f64).unwrap()
    );
    assert_eq!(
        0.0_f64,
        store.invoke_typed_without_fuel(function, -0.0_f64).unwrap()
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
    let mut store = Store::new(());
    let module = store
        .module_instantiate(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;

    let function = store
        .instance_export(module, "f64_ceil")
        .unwrap()
        .as_func()
        .unwrap();

    assert_eq!(
        2.0_f64,
        store.invoke_typed_without_fuel(function, 1.5_f64).unwrap()
    );
    assert_eq!(
        -1.0_f64,
        store.invoke_typed_without_fuel(function, -1.5_f64).unwrap()
    );
    assert_eq!(
        0.0_f64,
        store.invoke_typed_without_fuel(function, -0.1_f64).unwrap()
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
    let mut store = Store::new(());
    let module = store
        .module_instantiate(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;

    let function = store
        .instance_export(module, "f64_floor")
        .unwrap()
        .as_func()
        .unwrap();

    assert_eq!(
        1.0_f64,
        store.invoke_typed_without_fuel(function, 1.5_f64).unwrap()
    );
    assert_eq!(
        -2.0_f64,
        store.invoke_typed_without_fuel(function, -1.5_f64).unwrap()
    );
    assert_eq!(
        -1.0_f64,
        store.invoke_typed_without_fuel(function, -0.1_f64).unwrap()
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
    let mut store = Store::new(());
    let module = store
        .module_instantiate(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;

    let function = store
        .instance_export(module, "f64_trunc")
        .unwrap()
        .as_func()
        .unwrap();

    assert_eq!(
        1.0_f64,
        store.invoke_typed_without_fuel(function, 1.5_f64).unwrap()
    );
    assert_eq!(
        -1.0_f64,
        store.invoke_typed_without_fuel(function, -1.5_f64).unwrap()
    );
    assert_eq!(
        0.0_f64,
        store.invoke_typed_without_fuel(function, 0.9_f64).unwrap()
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
    let mut store = Store::new(());
    let module = store
        .module_instantiate(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;

    let function = store
        .instance_export(module, "f64_nearest")
        .unwrap()
        .as_func()
        .unwrap();

    assert_eq!(
        2.0_f64,
        store.invoke_typed_without_fuel(function, 1.5_f64).unwrap()
    );
    assert_eq!(
        -2.0_f64,
        store.invoke_typed_without_fuel(function, -1.5_f64).unwrap()
    );
    assert_eq!(
        1.0_f64,
        store.invoke_typed_without_fuel(function, 0.6_f64).unwrap()
    );
    assert_eq!(
        0.0_f64,
        store.invoke_typed_without_fuel(function, 0.4_f64).unwrap()
    );
}

/// A simple function to test the f64.sqrt implementation
#[test_log::test]
#[cfg_attr(miri, ignore)] // sqrt is not supported in miri
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
    let mut store = Store::new(());
    let module = store
        .module_instantiate(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;

    let function = store
        .instance_export(module, "f64_sqrt")
        .unwrap()
        .as_func()
        .unwrap();

    assert_eq!(
        2.0_f64,
        store.invoke_typed_without_fuel(function, 4.0_f64).unwrap()
    );
    assert_eq!(
        1.4142135623730951_f64,
        store.invoke_typed_without_fuel(function, 2.0_f64).unwrap()
    );
    assert!(store
        .invoke_typed_without_fuel::<f64, f64>(function, -f64::NAN)
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
    let mut store = Store::new(());
    let module = store
        .module_instantiate(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;

    let function = store
        .instance_export(module, "f64_add")
        .unwrap()
        .as_func()
        .unwrap();

    assert_eq!(
        3.0_f64,
        store
            .invoke_typed_without_fuel(function, (1.5_f64, 1.5_f64))
            .unwrap()
    );
    assert_eq!(
        -1.0_f64,
        store
            .invoke_typed_without_fuel(function, (1.0_f64, -2.0_f64))
            .unwrap()
    );
    assert_eq!(
        0.0_f64,
        store
            .invoke_typed_without_fuel(function, (0.1_f64, -0.1_f64))
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
    let mut store = Store::new(());
    let module = store
        .module_instantiate(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;

    let function = store
        .instance_export(module, "f64_sub")
        .unwrap()
        .as_func()
        .unwrap();

    assert_eq!(
        0.0_f64,
        store
            .invoke_typed_without_fuel(function, (1.5_f64, 1.5_f64))
            .unwrap()
    );
    assert_eq!(
        3.0_f64,
        store
            .invoke_typed_without_fuel(function, (1.0_f64, -2.0_f64))
            .unwrap()
    );
    assert_eq!(
        0.2_f64,
        store
            .invoke_typed_without_fuel(function, (0.1_f64, -0.1_f64))
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
    let mut store = Store::new(());
    let module = store
        .module_instantiate(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;

    let function = store
        .instance_export(module, "f64_mul")
        .unwrap()
        .as_func()
        .unwrap();

    assert_eq!(
        6.0_f64,
        store
            .invoke_typed_without_fuel(function, (2.0_f64, 3.0_f64))
            .unwrap()
    );
    assert_eq!(
        -4.0_f64,
        store
            .invoke_typed_without_fuel(function, (2.0_f64, -2.0_f64))
            .unwrap()
    );
    assert_eq!(
        0.0_f64,
        store
            .invoke_typed_without_fuel(function, (0.0_f64, 5.0_f64))
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
    let mut store = Store::new(());
    let module = store
        .module_instantiate(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;

    let function = store
        .instance_export(module, "f64_div")
        .unwrap()
        .as_func()
        .unwrap();

    assert_eq!(
        2.0_f64,
        store
            .invoke_typed_without_fuel(function, (6.0_f64, 3.0_f64))
            .unwrap()
    );
    assert_eq!(
        -1.0_f64,
        store
            .invoke_typed_without_fuel(function, (2.0_f64, -2.0_f64))
            .unwrap()
    );
    assert!(store
        .invoke_typed_without_fuel::<(f64, f64), f64>(function, (1.0_f64, 0.0_f64))
        .unwrap()
        .is_infinite());
    assert!(store
        .invoke_typed_without_fuel::<(f64, f64), f64>(function, (0.0_f64, 0.0_f64))
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
    let mut store = Store::new(());
    let module = store
        .module_instantiate(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;

    let function = store
        .instance_export(module, "f64_min")
        .unwrap()
        .as_func()
        .unwrap();

    {
        let result = store
            .invoke_typed_without_fuel::<(f64, f64), f64>(function, (f64::NAN, -f64::NAN))
            .unwrap();
        assert!(result.is_nan());
    }
    {
        let result = store
            .invoke_typed_without_fuel::<(f64, f64), f64>(function, (f64::NAN, f64::NAN))
            .unwrap();
        assert!(result.is_nan());
        assert!(result.is_sign_positive());
    }
    {
        let result = store
            .invoke_typed_without_fuel::<(f64, f64), f64>(
                function,
                (f64::INFINITY, f64::NEG_INFINITY),
            )
            .unwrap();
        assert!(result.is_infinite());
        assert!(result.is_sign_negative());
    }
    assert_eq!(
        42_f64,
        store
            .invoke_typed_without_fuel(function, (f64::INFINITY, 42_f64))
            .unwrap()
    );
    assert_eq!(
        -0_f64,
        store
            .invoke_typed_without_fuel(function, (-0_f64, 0_f64))
            .unwrap()
    );
    assert_eq!(
        1.0_f64,
        store
            .invoke_typed_without_fuel(function, (1.0_f64, 2.0_f64))
            .unwrap()
    );
    assert_eq!(
        -2.0_f64,
        store
            .invoke_typed_without_fuel(function, (-1.0_f64, -2.0_f64))
            .unwrap()
    );
    assert_eq!(
        -0.0_f64,
        store
            .invoke_typed_without_fuel(function, (0.0_f64, -0.0_f64))
            .unwrap()
    );
    assert!(store
        .invoke_typed_without_fuel::<(f64, f64), f64>(function, (f64::NAN, 1.0_f64))
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
    let mut store = Store::new(());
    let module = store
        .module_instantiate(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;

    let function = store
        .instance_export(module, "f64_max")
        .unwrap()
        .as_func()
        .unwrap();

    {
        let result = store
            .invoke_typed_without_fuel::<(f64, f64), f64>(function, (f64::NAN, -f64::NAN))
            .unwrap();
        assert!(result.is_nan());
    }
    {
        let result = store
            .invoke_typed_without_fuel::<(f64, f64), f64>(function, (f64::NAN, f64::NAN))
            .unwrap();
        assert!(result.is_nan());
        assert!(result.is_sign_positive());
    }
    {
        let result = store
            .invoke_typed_without_fuel::<(f64, f64), f64>(
                function,
                (f64::INFINITY, f64::NEG_INFINITY),
            )
            .unwrap();
        assert!(result.is_infinite());
        assert!(result.is_sign_positive());
    }
    assert_eq!(
        42_f64,
        store
            .invoke_typed_without_fuel(function, (f64::NEG_INFINITY, 42_f64))
            .unwrap()
    );
    assert_eq!(
        0_f64,
        store
            .invoke_typed_without_fuel(function, (-0_f64, 0_f64))
            .unwrap()
    );

    assert_eq!(
        2.0_f64,
        store
            .invoke_typed_without_fuel(function, (1.0_f64, 2.0_f64))
            .unwrap()
    );
    assert_eq!(
        -1.0_f64,
        store
            .invoke_typed_without_fuel(function, (-1.0_f64, -2.0_f64))
            .unwrap()
    );
    assert_eq!(
        0.0_f64,
        store
            .invoke_typed_without_fuel(function, (0.0_f64, -0.0_f64))
            .unwrap()
    );
    assert!(store
        .invoke_typed_without_fuel::<(f64, f64), f64>(function, (f64::NAN, 1.0_f64))
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
    let mut store = Store::new(());
    let module = store
        .module_instantiate(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;

    let function = store
        .instance_export(module, "f64_copysign")
        .unwrap()
        .as_func()
        .unwrap();

    assert_eq!(
        1.5_f64,
        store
            .invoke_typed_without_fuel(function, (1.5_f64, 2.0_f64))
            .unwrap()
    );
    assert_eq!(
        -1.5_f64,
        store
            .invoke_typed_without_fuel(function, (1.5_f64, -2.0_f64))
            .unwrap()
    );
    assert_eq!(
        -1.5_f64,
        store
            .invoke_typed_without_fuel(function, (-1.5_f64, -0.0_f64))
            .unwrap()
    );
    assert_eq!(
        1.5_f64,
        store
            .invoke_typed_without_fuel(function, (-1.5_f64, 0.0_f64))
            .unwrap()
    );
}
