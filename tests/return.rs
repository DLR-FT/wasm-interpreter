use wasm::DEFAULT_MODULE;

/// A simple function to add 2 two i32s but using the RETURN opcode.
#[test_log::test]
fn return_valid() {
    use wasm::{validate, RuntimeInstance};

    let wat = r#"
    (module
        (func (export "add") (param $x i32) (param $y i32) (result i32)
            local.get $x
            local.get $x
            local.get $x
            local.get $x
            local.get $x
            local.get $x
            local.get $x
            local.get $y
            i32.add
            return
        )
    )
    "#;
    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        12,
        instance
            .invoke_typed(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "add")
                    .unwrap(),
                (10, 2)
            )
            .unwrap()
    );
    assert_eq!(
        2,
        instance
            .invoke_typed(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "add")
                    .unwrap(),
                (0, 2)
            )
            .unwrap()
    );
    assert_eq!(
        -4,
        instance
            .invoke_typed(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "add")
                    .unwrap(),
                (-6, 2)
            )
            .unwrap()
    );
}
