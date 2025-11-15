use wasm::{
    checked::{StoredExternVal, StoredValue},
    validate, Store,
};

const MULTIPLY_WAT_TEMPLATE: &str = r#"
    (module
        (func (export "add_one") (param $x {{TYPE}}) (result {{TYPE}})
            local.get $x
            {{TYPE}}.const 1
            {{TYPE}}.add)
    )
"#;

/// A simple function to add 1 to an i32 and return the result
#[test_log::test]
fn i32_add_one() {
    let wat = String::from(MULTIPLY_WAT_TEMPLATE).replace("{{TYPE}}", "i32");
    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut store = Store::new(());
    let module = store
        .module_instantiate_checked(&validation_info, Vec::new(), None)
        .expect("instantiation to succeed");

    let Ok(StoredExternVal::Func(add_one)) = store.instance_export_checked(module, "add_one")
    else {
        panic!("expected function export");
    };

    assert_eq!(
        &[StoredValue::I32(12)],
        &*store
            .invoke_without_fuel_checked(add_one, vec![StoredValue::I32(11)])
            .unwrap()
    );
    assert_eq!(
        &[StoredValue::I32(1)],
        &*store
            .invoke_without_fuel_checked(add_one, vec![StoredValue::I32(0)])
            .unwrap()
    );
    assert_eq!(
        &[StoredValue::I32(-5_i32 as u32)],
        &*store
            .invoke_without_fuel_checked(add_one, vec![StoredValue::I32(-6_i32 as u32)])
            .unwrap()
    );
}

/// A simple function to add 1 to an i64 and return the result
#[test_log::test]
fn i64_add_one() {
    let wat = String::from(MULTIPLY_WAT_TEMPLATE).replace("{{TYPE}}", "i64");
    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut store = Store::new(());

    let module = store
        .module_instantiate_checked(&validation_info, Vec::new(), None)
        .expect("instantiation to succeed");

    let Ok(StoredExternVal::Func(add_one)) = store.instance_export_checked(module, "add_one")
    else {
        panic!("expected function export");
    };

    assert_eq!(
        &[StoredValue::I64(12)],
        &*store
            .invoke_without_fuel_checked(add_one, vec![StoredValue::I64(11)])
            .unwrap()
    );
    // WE WANT THIS: assert_eq!(12u64, store.invoke_typed_without_fuel(add_one, 11u64).unwrap());
    assert_eq!(
        &[StoredValue::I64(1)],
        &*store
            .invoke_without_fuel_checked(add_one, vec![StoredValue::I64(0)])
            .unwrap()
    );
    assert_eq!(
        &[StoredValue::I64(-5_i64 as u64)],
        &*store
            .invoke_without_fuel_checked(add_one, vec![StoredValue::I64(-6_i64 as u64)])
            .unwrap()
    );
}
