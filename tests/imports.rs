use wasm::{validate, RuntimeInstance};

const WAT: &'static str = r#"
    (module
        (import "func1" "add_two" (func $add_two (param i32 i32) (result i32)))
        (func (export "add_one") (param $x i32) (result i32)
            local.get $x
            i32.const 1
            i32.add)
    )
"#;

#[test_log::test]
fn test_imports_invoke_named() {
    let wasm_bytes = wat::parse_str(WAT).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(12, instance.invoke_named("add_one", 11).unwrap());
}

#[test_log::test]
fn test_imports_invoke_func() {
    let wasm_bytes = wat::parse_str(WAT).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(12, instance.invoke_func(1, 11).unwrap());
}
