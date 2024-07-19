use wasm::{validate, RuntimeInstance};

const MULTIPLY_WAT_TEMPLATE: &'static str = r#"
    (module
        (func (export "multiply_vec2") (param $x i32) (param $y i32) (result i32 i32)
            local.get $x
            i32.const 3
            i32.mul
            local.get $y
            i32.const 7
            i32.mul
        )
    )
"#;
/// A simple function to multiply two values by 3 and 7 and return the resulting values
#[test_log::test]
pub fn i32_multiple_return_values() {
    let wasm_bytes = wat::parse_str(MULTIPLY_WAT_TEMPLATE).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");

    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        (33, 7 * 17),
        instance.invoke_named("multiply_vec2", (11, 17)).unwrap()
    );
}
