use wasm::{validate, RuntimeInstance};

const FUNCTION_CALL: &'static str = r#"
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
        instance.invoke_named("simple_caller", (3, 7)).unwrap()
    );
}
