use wasm::{validate, Store};

const WAT_SUBTRACT_TEMPLATE: &str = r#"
    (module
        (func (export "subtract") (param $x {{TYPE}}) (param $y {{TYPE}}) (result {{TYPE}})
            local.get $x
            local.get $y
            {{TYPE}}.sub
        )
    )
"#;

/// A simple function to multiply by 3 a i64 value and return the result
#[test_log::test]
pub fn i64_subtract() {
    let wat = String::from(WAT_SUBTRACT_TEMPLATE).replace("{{TYPE}}", "i64");

    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");

    let mut store = Store::new(());
    let module = store
        .module_instantiate_unchecked(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;

    let subtract = store
        .instance_export_unchecked(module, "subtract")
        .unwrap()
        .as_func()
        .unwrap();

    assert_eq!(
        -10_i64,
        store
            .invoke_typed_without_fuel_unchecked(subtract, (1_i64, 11_i64))
            .unwrap()
    );
    assert_eq!(
        0_i64,
        store
            .invoke_typed_without_fuel_unchecked(subtract, (0_i64, 0_i64))
            .unwrap()
    );
    assert_eq!(
        10_i64,
        store
            .invoke_typed_without_fuel_unchecked(subtract, (-10_i64, -20_i64))
            .unwrap()
    );

    assert_eq!(
        i64::MAX - 1,
        store
            .invoke_typed_without_fuel_unchecked(subtract, (i64::MAX - 1, 0_i64))
            .unwrap()
    );
    assert_eq!(
        i64::MIN + 3,
        store
            .invoke_typed_without_fuel_unchecked(subtract, (i64::MIN + 3, 0_i64))
            .unwrap()
    );
}
