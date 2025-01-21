use wasm::DEFAULT_MODULE;

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
            i32.const 2
            i32.const 3
            i32.add
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
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    // Set global to 17. 5 is returned as previous (default) value.
    assert_eq!(
        5,
        instance
            .invoke(
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
            .invoke(
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
            i32.add
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
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    // Set global to 17. 3 is returned as previous (default) value.
    assert_eq!(
        3,
        instance
            .invoke(
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
            .invoke(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "get")
                    .unwrap(),
                ()
            )
            .unwrap()
    );
}

#[test_log::test]
fn global_f32_and_f64() {
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
            i32.add
        )
    )
    "#;
    let wasm_bytes = wat::parse_str(wat).unwrap();

    if validate(&wasm_bytes).is_ok() {
        panic!("validation succeeded")
    }
}
