use wasm::{validate, RuntimeInstance};
const BASE_WAT: &'static str = r#"
    (module
        (memory 1)
        (func (export "store_num") (param $x {{TYPE}})
            ({{TYPE}}.store (i32.const 0) (local.get $x))
        )
        (func (export "load_num") (result {{TYPE}})
            ({{TYPE}}.load (i32.const 0))
        )
    )
"#;
/// Two simple methods for storing and loading an i32 from the first slot in linear memory.
#[test_log::test]
fn basic_memory() {
    let wat = String::from(BASE_WAT).replace("{{TYPE}}", "i32");
    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let _ = instance.invoke_named::<i32, ()>("store_num", 42);
    assert_eq!(42, instance.invoke_named("load_num", ()).unwrap());
}

/// Two simple methods for storing and loading an f32 from the first slot in linear memory.
#[test_log::test]
fn f32_basic_memory() {
    let wat = String::from(BASE_WAT).replace("{{TYPE}}", "f32");
    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    instance.invoke_func::<f32, ()>(0, 133.7_f32).unwrap();
    assert_eq!(133.7_f32, instance.invoke_func(1, ()).unwrap());
}
