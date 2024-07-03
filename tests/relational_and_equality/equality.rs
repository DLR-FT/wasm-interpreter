use wasm::{validate, RuntimeInstance};
const BASE_WAT_1_ARG: &'static str = r#"
    (module
        (func (export "template") (param $x i32) (result i32)
            local.get $x
            i32.{{0}})
    )
    "#;

/// A simple function to test the i32.eqz function
#[test_log::test]
pub fn i32_eqz() {
    let wat = String::from(BASE_WAT_1_ARG).replace("{{0}}", "eqz");

    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");

    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(1, instance.invoke_func(0, 0));
    assert_eq!(0, instance.invoke_func(0, 9001));
}
