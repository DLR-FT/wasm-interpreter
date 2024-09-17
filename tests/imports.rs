use wasm::{validate, RuntimeInstance};

const UNUSED_IMPORTS: &str = r#"
(module
    (import "env" "dummy1" (func (param i32)))
    (import "env" "dummy2" (func (param i32)))
    (func (export "get_three") (param) (result i32)
        i32.const 1
        i32.const 2
        i32.add
    )
)"#;

const SIMPLE_IMPORT: &str = r#"
(module
    (import "env" "print" (func $print (param i32)))
    (func (export "print_three")
        i32.const 1
        i32.const 2
        i32.add
        call $print
    )
)"#;

/// This test checks that the import order is correct, even if the imports are not used.
#[test_log::test]
pub fn import_order() {
    let wasm_bytes = wat::parse_str(UNUSED_IMPORTS).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(3, instance.invoke_named("get_three", ()).unwrap());
    // Function 0 should be the imported function "dummy1"
    // Function 1 should be the imported function "dummy2"
    // Function 2 should be the local function "get_three"
    assert_eq!(3, instance.invoke_func(2, ()).unwrap());
}

#[test_log::test]
pub fn compile_simple_import() {
    let wasm_bytes = wat::parse_str(SIMPLE_IMPORT).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let _ = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    // assert_eq!((), instance.invoke_named("print_three", ()).unwrap());
    // Function 0 should be the imported function
    // assert_eq!((), instance.invoke_func(1, ()).unwrap());
}

#[test_log::test]
pub fn run_simple_import() {
    let wasm_bytes = wat::parse_str(SIMPLE_IMPORT).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!((), instance.invoke_named("print_three", ()).unwrap());
    // Function 0 should be the imported function
    assert_eq!((), instance.invoke_func(1, ()).unwrap());
}
