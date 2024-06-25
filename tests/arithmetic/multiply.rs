/// A simple function to multiply by 2 a i32 value and return the result
#[test_log::test]
pub fn multiply() {
    use wasm::{validate, RuntimeInstance};

    let wat = r#"
    (module
        (func (export "multiply") (param $x i32) (result i32)
            local.get $x
            i32.const 2
            i32.mul)
    )
    "#;

    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");
    
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(22, instance.invoke_func(0, 11));
    assert_eq!(0, instance.invoke_func(0, 0));
    assert_eq!(-20, instance.invoke_func(0, -10));
}
