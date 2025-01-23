use wasm::{validate, RuntimeError, RuntimeInstance, DEFAULT_MODULE};

const UNMET_IMPORTS: &str = r#"
(module
    (import "env" "dummy1" (func (param i32)))
    (import "env" "dummy2" (func (param i32)))
    (func (export "get_three") (param) (result i32)
        i32.const 1
        i32.const 2
        i32.add
    )
)"#;

const SIMPLE_IMPORT_BASE: &str = r#"
(module
    (import "env" "get_one" (func $get_one (param) (result i32)))
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
pub fn unmet_imports() {
    let wasm_bytes = wat::parse_str(UNMET_IMPORTS).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let get_three = instance
        .get_function_by_name(DEFAULT_MODULE, "get_three")
        .unwrap();

    assert_eq!(
        RuntimeError::UnmetImport,
        instance
            .invoke::<(), i32>(&get_three, ())
            .expect_err("Expected invoke to fail due to unmet imports")
    );
}

#[test_log::test]
pub fn compile_simple_import() {
    let wasm_bytes = wat::parse_str(SIMPLE_IMPORT_BASE).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance =
        RuntimeInstance::new_named("base", &validation_info).expect("instantiation failed");

    let wasm_bytes = wat::parse_str(SIMPLE_IMPORT_ADDON).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    instance
        .add_module("addon", &validation_info)
        .expect("Successful instantiation");

    // assert_eq!((), instance.invoke_named("print_three", ()).unwrap());
    // Function 0 should be the imported function
    // assert_eq!((), instance.invoke_func(1, ()).unwrap());
}

#[test_log::test]
pub fn run_simple_import() {
    let wasm_bytes = wat::parse_str(SIMPLE_IMPORT_BASE).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance =
        RuntimeInstance::new_named("base", &validation_info).expect("instantiation failed");

    let wasm_bytes = wat::parse_str(SIMPLE_IMPORT_ADDON).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    instance
        .add_module("env", &validation_info)
        .expect("Successful instantiation");

    let get_three = instance.get_function_by_name("base", "get_three").unwrap();
    assert_eq!(3, instance.invoke(&get_three, ()).unwrap());

    // Function 0 should be the imported function
    let get_three = instance.get_function_by_index(0, 1).unwrap();
    assert_eq!(3, instance.invoke(&get_three, ()).unwrap());
}
