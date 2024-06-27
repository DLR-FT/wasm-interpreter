/// A function that tests the FC multibyte instructions
/// https://pengowray.github.io/wasm-ops/#fc
///
/// At the moment, it dies on validation, as the f32 data type is not implemented
#[test_log::test]
#[should_panic(expected = "i32.trunc_sat_f32_s NOT implemented")]
fn panic_on_not_implemented() {
    use wasm::validate;

    // invalid wasm code
    // only for testing op codes enum
    let wat = r#"
    (module
        (func (export "fc_extensions") (result i32)
            i32.trunc_sat_f32_s)
    )
    "#;
    let wasm_bytes = wat::parse_str(wat).unwrap();

    validate(&wasm_bytes).unwrap();
}

#[test_log::test]
#[should_panic(expected = "not implemented: Unimplemented instruction for bytes: 0xFC 0x1")]
fn fc_extensions() {
    use wasm::validate;

    // invalid wasm code
    // only for testing op codes enum
    let wat = r#"
  (module
      (func (export "fc_extensions") (result i32)
          i32.trunc_sat_f32_u)
  )
  "#;
    let wasm_bytes = wat::parse_str(wat).unwrap();

    validate(&wasm_bytes).unwrap();
}
