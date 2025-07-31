use wasm::{validate, RuntimeInstance, Value};

#[test_log::test]
pub fn host_func_call_within_module() {
    let wat = r#"(module
    (import "hello_mod" "hello" (func $hello (param) (result)))
    (func (export "hello_caller") (param i32) (result i32)
        local.get 0
        i32.const 2
        call $hello
        i32.add
    )
)"#;
    let wasm_bytes = wat::parse_str(wat).unwrap();
    fn hello(_values: Vec<Value>) -> Vec<Value> {
        println!("Host function says hello from wasm!");
        Vec::new()
    }

    let mut runtime_instance = RuntimeInstance::new();
    runtime_instance
        .add_host_function_typed::<(), ()>("hello_mod", "hello", hello)
        .expect("function registration failed");
    runtime_instance
        .add_module(
            "importing_mod",
            &validate(&wasm_bytes).expect("validation failed"),
        )
        .expect("instantiation failed");
    let function_ref = runtime_instance
        .get_function_by_name("importing_mod", "hello_caller")
        .expect("wasm function could not be found");
    let result = runtime_instance
        .invoke_typed::<i32, i32>(&function_ref, 2)
        .expect("wasm function invocation failed");
    assert_eq!(4, result);
}
