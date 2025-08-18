use wasm::{validate, Store};

#[test_log::test]

fn out_of_fuel() {
    let wat = r#"
            (module
            (func (export "loop_forever") (loop br 0)
            ))"#;
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = &validate(&wasm_bytes).expect("validation failed");
    let mut store = Store::new((), Some(50));
    store.add_module("loop_forever", validation_info).unwrap();
    assert_eq!(store.invoke(0, vec![]), Err(wasm::RuntimeError::OutOfFuel));
}
