use log::info;
use wasm::{RuntimeError, TrapError};

#[test_log::test]
pub fn runtime_error_bad_conversion_to_integer() {
    info!("{}", RuntimeError::Trap(TrapError::BadConversionToInteger))
}
