/// Two simple methods for storing and loading an i32 from the first slot in linear memory.
#[test_log::test]
fn basic_memory() {
    use wasm::{validate, RuntimeInstance};

    let wat = r#"
    (module
        (memory 1)
        (func (export "store_num") (param $x i32)
            i32.const 0
            local.get $x
            i32.store)
        (func (export "load_num") (result i32)
            i32.const 0
            i32.load)
    )
    "#;
    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let _ = instance.invoke_func::<i32, ()>(0, 42);
    assert_eq!(42, instance.invoke_func(1, ()).unwrap());
}
