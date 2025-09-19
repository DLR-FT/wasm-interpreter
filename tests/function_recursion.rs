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
    let mut instance = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");

    assert_eq!(
        3 * 7 + 13,
        instance
            .invoke_typed(
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
    let mut instance = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");

    assert_eq!(
        12,
        instance
            .invoke_typed(
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
            .invoke_typed(
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
            .invoke_typed(
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
    use wasm::{validate, ValidationError};

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
        matches!(
            validate(&wasm_bytes),
            Err(ValidationError::EndInvalidValueStack)
        ),
        "validation incorrectly passed"
    );
}

#[test_log::test]
fn multivalue_call() {
    let wat = r#"
    (module
        (func $foo (param $x i64) (param $y i32) (param $z f32) (result i32 f32 i64)
            local.get $y
            local.get $z
            local.get $x
        )
        (func $bar (export "bar") (result i32 f32 i64)
            i64.const 5
            i32.const 10
            f32.const 42.0
            call $foo
        )
    )
    "#;
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");

    let foo_fn = instance
        .get_function_by_name(DEFAULT_MODULE, "bar")
        .unwrap();

    assert_eq!(
        (10, 42.0, 5),
        instance
            .invoke_typed::<(), (i32, f32, i64)>(&foo_fn, ())
            .unwrap()
    );
}
