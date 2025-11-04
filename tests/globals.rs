use wasm::{ExternVal, GlobalType, NumType, ValType, Value, DEFAULT_MODULE};

/// The WASM program has one mutable global initialized with a constant 3.
/// It exports two methods:
///  - Setting the global's value and returning its previous value
///  - Getting the global's current value
#[test_log::test]
fn valid_global() {
    use wasm::{validate, RuntimeInstance};

    let wat = r#"
    (module
        (global $my_global (mut i32)
            i32.const 5
        )

        ;; Set global to a value and return the previous one
        (func (export "set") (param i32) (result i32)
            global.get $my_global
            local.get 0
            global.set $my_global)

        ;; Returns the global's current value
        (func (export "get") (result i32)
            global.get $my_global)
    )
    "#;
    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let (mut instance, _default_module) =
        RuntimeInstance::new_with_default_module((), &validation_info)
            .expect("instantiation failed");

    // Set global to 17. 5 is returned as previous (default) value.
    assert_eq!(
        5,
        instance
            .invoke_typed(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "set")
                    .unwrap(),
                17
            )
            .unwrap()
    );

    // Now 17 will be returned when getting the global
    assert_eq!(
        17,
        instance
            .invoke_typed(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "get")
                    .unwrap(),
                ()
            )
            .unwrap()
    );
}

#[test_log::test]
fn global_invalid_value_stack() {
    use wasm::validate;

    let wat = r#"
    (module
        (global $my_global (mut i32)
            i32.const 2
            i32.const 2
            i32.const 2
            i32.const 2
            i32.const 2
            i32.const 2
            i32.const 3
        )
    )
    "#;
    let wasm_bytes = wat::parse_str(wat).unwrap();

    if validate(&wasm_bytes).is_ok() {
        panic!("validation succeeded")
    }
}

#[ignore = "not yet implemented"]
#[test_log::test]
fn imported_globals() {
    use wasm::{validate, RuntimeInstance};

    let wat = r#"
    (module
        (import "env" "global" (global $my_global (mut i32)))

        ;; Set global to a value and return the previous one
        (func (export "set") (param i32) (result i32)
            global.get $my_global
            local.get 0
            global.set $my_global)

        ;; Returns the global's current value
        (func (export "get") (result i32)
            global.get $my_global)
    )
    "#;
    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let (mut instance, _default_module) =
        RuntimeInstance::new_with_default_module((), &validation_info)
            .expect("instantiation failed");

    // Set global to 17. 3 is returned as previous (default) value.
    assert_eq!(
        3,
        instance
            .invoke_typed(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "set")
                    .unwrap(),
                17
            )
            .unwrap()
    );

    // Now 17 will be returned when getting the global
    assert_eq!(
        17,
        instance
            .invoke_typed(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "get")
                    .unwrap(),
                ()
            )
            .unwrap()
    );
}

#[test_log::test]
fn global_invalid_instr() {
    use wasm::validate;

    let wat = r#"
    (module
        (global $my_global (mut i32)
            i32.const 2
            i32.const 2
            i32.const 2
            i32.add
            i32.const 2
            i32.const 2
            i32.const 2
            i32.const 3

        )
    )
    "#;
    let wasm_bytes = wat::parse_str(wat).unwrap();

    if validate(&wasm_bytes).is_ok() {
        panic!("validation succeeded")
    }
}

#[test_log::test]
fn embedder_interface() {
    use wasm::{validate, RuntimeInstance};

    let wat = r#"
    (module
        (global (export "global_0") (mut i32) i32.const 1)
        (global (export "global_1") (mut i64) i64.const 3)
    )
    "#;
    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let (mut instance, module) = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");

    let ExternVal::Global(global_0) = instance.store.instance_export(module, "global_0").unwrap()
    else {
        panic!("expected global");
    };
    let ExternVal::Global(global_1) = instance.store.instance_export(module, "global_1").unwrap()
    else {
        panic!("expected global");
    };

    assert_eq!(instance.store.global_read(global_0), Value::I32(1));
    assert_eq!(instance.store.global_read(global_1), Value::I64(3));

    assert_eq!(
        instance.store.global_write(global_0, Value::I32(33)),
        Ok(())
    );

    assert_eq!(instance.store.global_read(global_0), Value::I32(33));
    assert_eq!(instance.store.global_read(global_1), Value::I64(3));

    assert_eq!(
        instance.store.global_type(global_0),
        GlobalType {
            ty: ValType::NumType(NumType::I32),
            is_mut: true,
        }
    );

    assert_eq!(
        instance.store.global_type(global_1),
        GlobalType {
            ty: ValType::NumType(NumType::I64),
            is_mut: true,
        }
    );
}
