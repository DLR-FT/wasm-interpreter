use wasm::{validate, ExternVal, RuntimeInstance, Store};

const SIMPLE_IMPORT_BASE: &str = r#"
(module
    (import "addon" "get_one" (func $get_one (param) (result i32)))
    (func (export "get_three") (param) (result i32)
        call $get_one
        i32.const 2
        i32.add
    )
)"#;

const SIMPLE_IMPORT_ADDON: &str = r#"
(module
    (func (export "get_one") (param) (result i32)
        i32.const 1
    )
)"#;

#[test_log::test]
pub fn compile_simple_import() {
    let mut store: Store = Default::default();
    // let mut linker: Linker = Default::default();

    let wasm_bytes_addon = wat::parse_str(SIMPLE_IMPORT_ADDON).unwrap();
    let validation_info_addon = validate(&wasm_bytes_addon).expect("validation failed");

    let res = store.add_module("addon", &validation_info_addon);
    if res.is_err() {
        // println!("{:#?}", res.unwrap_err());
        panic!("{}", res.unwrap_err());
    }

    let wasm_bytes_base = wat::parse_str(SIMPLE_IMPORT_BASE).unwrap();
    let validation_info_base = validate(&wasm_bytes_base).expect("validation failed");

    store.add_module("base", &validation_info_base).unwrap();
    // let mut instance_base = linker
    //     .instantiate(&mut store, &validation_info_base)
    //     .unwrap();
    // let mut instance =
    //     RuntimeInstance::new_named("base", &validation_info_base).expect("instantiation failed");

    let &ExternVal::Func(func_addr) = store
        .registry
        .lookup("base".into(), "get_three".into())
        .unwrap()
    else {
        panic!("this entity is not a function")
    };

    println!("{:#?}", store.invoke::<(), i32>(func_addr, ()).unwrap());

    // let mut instance_addon = linker
    //     .instantiate(&mut store, &validation_info_addon)
    //     .unwrap();

    // instance
    //     .add_module("addon", &validation_info)
    // .expect("Successful instantiation");
}

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
    let wat_dummy = "(module)";
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let wasm_dummy = wat::parse_str(wat_dummy).unwrap();

    fn hello() {
        println!("Host function says hello from wasm!");
    }

    let mut runtime_instance = RuntimeInstance::new(&validate(&wasm_dummy).unwrap()).unwrap();
    runtime_instance
        .add_host_function("hello_mod", "hello", hello)
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
        .invoke::<i32, i32>(&function_ref, 2)
        .expect("wasm function invocation failed");
    assert_eq!(4, result);
}
