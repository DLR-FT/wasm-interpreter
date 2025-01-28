use wasm::{validate, RuntimeInstance, DEFAULT_MODULE};

const FUNCTION_CALL: &str = r#"
    (module
        (func (export "simple_caller") (param $x i32) (param $y i32) (result i32)
            (call $callee (i32.mul (local.get $x) (local.get $y)))
        )
        (func $callee (param $x i32) (result i32)
            local.get $x
            i32.const 13
            i32.add
        )
    )
"#;

/// A simple function to multiply two numbers, then calling another function to add 13
#[test_log::test]
fn simple_function_call() {
    let wasm_bytes = wat::parse_str(FUNCTION_CALL).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        3 * 7 + 13,
        instance
            .invoke(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "simple_caller")
                    .unwrap(),
                (3, 7)
            )
            .unwrap()
    );
}

/// A simple function to add 2 to an i32 using a recusive call to "add_one" and return the result
#[test_log::test]
fn recursion_valid() {
    use wasm::{validate, RuntimeInstance};

    let wat = r#"
    (module
        (func $add_one (export "add_one") (param $x i32) (result i32)
            local.get $x
            i32.const 1
            i32.add
        )
        (func (export "add_two") (param $x i32) (result i32)
            local.get $x
            call $add_one
            call $add_one
        )
    )
    "#;
    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        12,
        instance
            .invoke(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "add_two")
                    .unwrap(),
                10
            )
            .unwrap()
    );
    assert_eq!(
        2,
        instance
            .invoke(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "add_two")
                    .unwrap(),
                0
            )
            .unwrap()
    );
    assert_eq!(
        -4,
        instance
            .invoke(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "add_two")
                    .unwrap(),
                -6
            )
            .unwrap()
    );
}

#[test_log::test]
fn recursion_busted_stack() {
    use wasm::{validate, Error};

    let wat = r#"
    (module
        (func $add_one (export "add_one") (param $x i32) (result i32 i32)
            local.get $x
            i32.const 1
            i32.add
            local.get $x
            i32.const 1
            i32.add
        )
        (func (export "add_two") (param $x i32) (result i32)
            local.get $x
            call $add_one
            call $add_one
        )
    )
    "#;
    let wasm_bytes = wat::parse_str(wat).unwrap();

    assert!(
        matches!(validate(&wasm_bytes), Err(Error::EndInvalidValueStack)),
        "validation incorrectly passed"
    );
}
