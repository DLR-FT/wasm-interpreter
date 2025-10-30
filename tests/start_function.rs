//! The WASM program stores 42 into linear memory upon instantiation through a start function.
//! Then it reads the same value and checks its value.

use wasm::DEFAULT_MODULE;
#[test_log::test]
fn start_function() {
    use wasm::{validate, RuntimeInstance};

    let wat = r#"
    (module
        (memory 1)

        (func $store42
            i32.const 0
            i32.const 42
            i32.store)

        (start $store42)

        (func (export "load_num") (result i32)
            i32.const 0
            i32.load)
    )
    "#;
    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let (mut instance, _default_module) =
        RuntimeInstance::new_with_default_module((), &validation_info)
            .expect("instantiation failed");

    assert_eq!(
        42,
        instance
            .invoke_typed(
                &instance
                    .get_function_by_name(DEFAULT_MODULE, "load_num")
                    .unwrap(),
                ()
            )
            .unwrap()
    );
}
