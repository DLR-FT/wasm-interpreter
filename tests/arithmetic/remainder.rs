use wasm::{validate, Store};
use wasm::{RuntimeError, TrapError};
const REM_S_WAT: &str = r#"
    (module
        (func (export "rem_s") (param $divisor {{TYPE}}) (param $dividend {{TYPE}}) (result {{TYPE}})
            local.get $divisor
            local.get $dividend
            {{TYPE}}.rem_s)
    )
"#;

const REM_U_WAT: &str = r#"
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
    let mut store = Store::new(());
    let module = store
        .module_instantiate_unchecked(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;

    let rem_s = store
        .instance_export_unchecked(module, "rem_s")
        .unwrap()
        .as_func()
        .unwrap();

    assert_eq!(
        0_i64,
        store
            .invoke_typed_without_fuel_unchecked(rem_s, (20_i64, 2_i64))
            .unwrap()
    );
    assert_eq!(
        999_i64,
        store
            .invoke_typed_without_fuel_unchecked(rem_s, (10_000_i64, 9_001_i64))
            .unwrap()
    );
    assert_eq!(
        -2_i64,
        store
            .invoke_typed_without_fuel_unchecked(rem_s, (-20_i64, 3_i64))
            .unwrap()
    );
    assert_eq!(
        -2_i64,
        store
            .invoke_typed_without_fuel_unchecked(rem_s, (-20_i64, -3_i64))
            .unwrap()
    );
    assert_eq!(
        2_i64,
        store
            .invoke_typed_without_fuel_unchecked(rem_s, (20_i64, -3_i64))
            .unwrap()
    );
    assert_eq!(
        2_i64,
        store
            .invoke_typed_without_fuel_unchecked(rem_s, (20_i64, 3_i64))
            .unwrap()
    );
    assert_eq!(
        0_i64,
        store
            .invoke_typed_without_fuel_unchecked(rem_s, (i64::MIN, -1_i64))
            .unwrap()
    );
    assert_eq!(
        0_i64,
        store
            .invoke_typed_without_fuel_unchecked(rem_s, (i64::MIN, 2_i64))
            .unwrap()
    );
}

/// A simple function to test i64 signed remainder's RuntimeError when dividing by 0
#[test_log::test]
pub fn i64_remainder_signed_panic_dividend_0() {
    let wat = String::from(REM_S_WAT).replace("{{TYPE}}", "i64");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut store = Store::new(());
    let module = store
        .module_instantiate_unchecked(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;

    let rem_s = store
        .instance_export_unchecked(module, "rem_s")
        .unwrap()
        .as_func()
        .unwrap();

    let result =
        store.invoke_typed_without_fuel_unchecked::<(i64, i64), i64>(rem_s, (222_i64, 0_i64));

    assert_eq!(
        result.unwrap_err(),
        RuntimeError::Trap(TrapError::DivideBy0)
    );
}

/// A simple function to test i64 unsigned remainder
#[test_log::test]
pub fn i64_remainder_unsigned_simple() {
    let wat = String::from(REM_U_WAT).replace("{{TYPE}}", "i64");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut store = Store::new(());
    let module = store
        .module_instantiate_unchecked(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;

    let rem_u = store
        .instance_export_unchecked(module, "rem_u")
        .unwrap()
        .as_func()
        .unwrap();

    assert_eq!(
        0_i64,
        store
            .invoke_typed_without_fuel_unchecked(rem_u, (i64::MIN, 2_i64))
            .unwrap()
    );
    assert_eq!(
        i64::MIN,
        store
            .invoke_typed_without_fuel_unchecked(rem_u, (i64::MIN, -2_i64))
            .unwrap()
    );
    assert_eq!(
        (i64::MAX - 1),
        store
            .invoke_typed_without_fuel_unchecked(rem_u, (-2_i64, i64::MIN))
            .unwrap()
    );
    assert_eq!(
        2_i64,
        store
            .invoke_typed_without_fuel_unchecked(rem_u, (2_i64, i64::MIN))
            .unwrap()
    );

    assert_eq!(
        0_i64,
        store
            .invoke_typed_without_fuel_unchecked(rem_u, (20_i64, 2_i64))
            .unwrap()
    );
    assert_eq!(
        999_i64,
        store
            .invoke_typed_without_fuel_unchecked(rem_u, (10_000_i64, 9_001_i64))
            .unwrap()
    );
    assert_eq!(
        2_i64,
        store
            .invoke_typed_without_fuel_unchecked(rem_u, (-20_i64, 3_i64))
            .unwrap()
    );
    assert_eq!(
        -20_i64,
        store
            .invoke_typed_without_fuel_unchecked(rem_u, (-20_i64, -3_i64))
            .unwrap()
    );
    assert_eq!(
        20_i64,
        store
            .invoke_typed_without_fuel_unchecked(rem_u, (20_i64, -3_i64))
            .unwrap()
    );
    assert_eq!(
        2_i64,
        store
            .invoke_typed_without_fuel_unchecked(rem_u, (20_i64, 3_i64))
            .unwrap()
    );
    assert_eq!(
        i64::MIN,
        store
            .invoke_typed_without_fuel_unchecked(rem_u, (i64::MIN, -1_i64))
            .unwrap()
    );
    assert_eq!(
        0_i64,
        store
            .invoke_typed_without_fuel_unchecked(rem_u, (i64::MIN, 2_i64))
            .unwrap()
    );
}

/// A simple function to test i64 signed remainder's RuntimeError when dividing by 0
#[test_log::test]
pub fn i64_remainder_unsigned_panic_dividend_0() {
    let wat = String::from(REM_U_WAT).replace("{{TYPE}}", "i64");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut store = Store::new(());
    let module = store
        .module_instantiate_unchecked(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;

    let rem_u = store
        .instance_export_unchecked(module, "rem_u")
        .unwrap()
        .as_func()
        .unwrap();

    let result = store.invoke_typed_without_fuel_unchecked::<(i64, i64), i64>(rem_u, (222, 0));

    assert_eq!(
        result.unwrap_err(),
        RuntimeError::Trap(TrapError::DivideBy0)
    );
}

/// A simple function to test signed remainder
#[test_log::test]
pub fn i32_remainder_signed_simple() {
    let wat = String::from(REM_S_WAT).replace("{{TYPE}}", "i32");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut store = Store::new(());
    let module = store
        .module_instantiate_unchecked(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;

    let rem_s = store
        .instance_export_unchecked(module, "rem_s")
        .unwrap()
        .as_func()
        .unwrap();

    assert_eq!(
        0,
        store
            .invoke_typed_without_fuel_unchecked(rem_s, (20, 2))
            .unwrap()
    );
    assert_eq!(
        999,
        store
            .invoke_typed_without_fuel_unchecked(rem_s, (10_000, 9_001))
            .unwrap()
    );
    assert_eq!(
        -2,
        store
            .invoke_typed_without_fuel_unchecked(rem_s, (-20, 3))
            .unwrap()
    );
    assert_eq!(
        -2,
        store
            .invoke_typed_without_fuel_unchecked(rem_s, (-20, -3))
            .unwrap()
    );
    assert_eq!(
        2,
        store
            .invoke_typed_without_fuel_unchecked(rem_s, (20, -3))
            .unwrap()
    );
    assert_eq!(
        2,
        store
            .invoke_typed_without_fuel_unchecked(rem_s, (20, 3))
            .unwrap()
    );
    assert_eq!(
        0,
        store
            .invoke_typed_without_fuel_unchecked(rem_s, (i32::MIN, -1))
            .unwrap()
    );
    assert_eq!(
        0,
        store
            .invoke_typed_without_fuel_unchecked(rem_s, (i32::MIN, 2))
            .unwrap()
    );
}

/// A simple function to test signed remainder's RuntimeError when dividing by 0
#[test_log::test]
pub fn remainder_signed_panic_dividend_0() {
    let wat = String::from(REM_S_WAT).replace("{{TYPE}}", "i32");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut store = Store::new(());
    let module = store
        .module_instantiate_unchecked(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;

    let rem_s = store
        .instance_export_unchecked(module, "rem_s")
        .unwrap()
        .as_func()
        .unwrap();

    let result = store.invoke_typed_without_fuel_unchecked::<(i32, i32), i32>(rem_s, (222, 0));

    assert_eq!(
        result.unwrap_err(),
        RuntimeError::Trap(TrapError::DivideBy0)
    );
}

/// A simple function to test unsigned remainder
#[test_log::test]
pub fn i32_remainder_unsigned_simple() {
    let wat = String::from(REM_U_WAT).replace("{{TYPE}}", "i32");
    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut store = Store::new(());
    let module = store
        .module_instantiate_unchecked(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;

    let rem_u = store
        .instance_export_unchecked(module, "rem_u")
        .unwrap()
        .as_func()
        .unwrap();

    assert_eq!(
        0,
        store
            .invoke_typed_without_fuel_unchecked(rem_u, (i32::MIN, 2))
            .unwrap()
    );
    assert_eq!(
        i32::MIN,
        store
            .invoke_typed_without_fuel_unchecked(rem_u, (i32::MIN, -2))
            .unwrap()
    );
    assert_eq!(
        -(i32::MIN + 2),
        store
            .invoke_typed_without_fuel_unchecked(rem_u, (-2, i32::MIN))
            .unwrap()
    );
    assert_eq!(
        2,
        store
            .invoke_typed_without_fuel_unchecked(rem_u, (2, i32::MIN))
            .unwrap()
    );
    assert_eq!(
        i32::MAX,
        store
            .invoke_typed_without_fuel_unchecked(rem_u, (i32::MAX, i32::MIN))
            .unwrap()
    );

    assert_eq!(
        0,
        store
            .invoke_typed_without_fuel_unchecked(rem_u, (20, 2))
            .unwrap()
    );
    assert_eq!(
        999,
        store
            .invoke_typed_without_fuel_unchecked(rem_u, (10_000, 9_001))
            .unwrap()
    );
    assert_eq!(
        2,
        store
            .invoke_typed_without_fuel_unchecked(rem_u, (-20, 3))
            .unwrap()
    );
    assert_eq!(
        -20,
        store
            .invoke_typed_without_fuel_unchecked(rem_u, (-20, -3))
            .unwrap()
    );
    assert_eq!(
        20,
        store
            .invoke_typed_without_fuel_unchecked(rem_u, (20, -3))
            .unwrap()
    );
    assert_eq!(
        2,
        store
            .invoke_typed_without_fuel_unchecked(rem_u, (20, 3))
            .unwrap()
    );
    assert_eq!(
        i32::MIN,
        store
            .invoke_typed_without_fuel_unchecked(rem_u, (i32::MIN, -1))
            .unwrap()
    );
    assert_eq!(
        0,
        store
            .invoke_typed_without_fuel_unchecked(rem_u, (i32::MIN, 2))
            .unwrap()
    );
}

/// A simple function to test signed remainder's RuntimeError when dividing by 0
#[test_log::test]
pub fn i32_remainder_unsigned_panic_dividend_0() {
    let wat = String::from(REM_U_WAT).replace("{{TYPE}}", "i32");
    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut store = Store::new(());
    let module = store
        .module_instantiate_unchecked(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;

    let rem_u = store
        .instance_export_unchecked(module, "rem_u")
        .unwrap()
        .as_func()
        .unwrap();

    let result = store.invoke_typed_without_fuel_unchecked::<(i32, i32), i32>(rem_u, (222, 0));

    assert_eq!(
        result.unwrap_err(),
        RuntimeError::Trap(TrapError::DivideBy0)
    );
}
