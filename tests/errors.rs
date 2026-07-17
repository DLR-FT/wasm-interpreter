use dlr_wasm_interpreter::{RuntimeError, TrapError};
use log::info;

#[test_log::test]
pub fn runtime_error_bad_conversion_to_integer() {
    info!("{}", RuntimeError::Trap(TrapError::BadConversionToInteger))
}
