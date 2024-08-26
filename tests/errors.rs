use wasm::RuntimeError;

#[test_log::test]
pub fn runtime_error_bad_conversion_to_integer() {
    println!("{}", RuntimeError::BadConversionToInteger)
}
