/*
# This file incorporates code from Wasmtime, originally
# available at https://github.com/bytecodealliance/wasmtime.
#
# The original code is licensed under the Apache License, Version 2.0
# (the "License"); you may not use this file except in compliance
# with the License. You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
*/

use std::error::Error;

use wasm::{
    value::{F32, F64},
    Value,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AssertEqError {
    left: String,
    right: String,
}

impl AssertEqError {
    pub fn assert_eq(left: Vec<Value>, right: Vec<Value>) -> Result<(), Self> {
        if left.len() != right.len() {
            return Err(AssertEqError {
                left: format!("Arr<len: {}>", left.len()),
                right: format!("Arr<len: {}>", right.len()),
            });
        }

        for i in 0..left.len() {
            let left_el = left[i];
            let right_el = right[i];

            match (left_el, right_el) {
                (Value::F32(a), Value::F32(b)) => {
                    match_f32(a, b)?;
                    Ok(())
                }
                (Value::F64(a), Value::F64(b)) => {
                    match_f64(a, b)?;
                    Ok(())
                }
                (a, b) => {
                    if a != b {
                        Err(AssertEqError {
                            left: format!("{:?}", left),
                            right: format!("{:?}", right),
                        })
                    } else {
                        Ok(())
                    }
                }
            }?;
        }
        Ok(())
    }
}

fn match_f32(actual: F32, expected: F32) -> Result<(), AssertEqError> {
    use wast::core::NanPattern;

    let actual_bits = actual.to_bits();

    let expected_nan_pattern: NanPattern<u32> = match expected.to_bits() {
        0x7fc0_0000 => NanPattern::CanonicalNan,
        // Inf and ArithmeticNan might overlap, and since we cast to our Value type and then back to NanPattern we need the distinction
        0x7f80_0001 => NanPattern::ArithmeticNan,
        val => NanPattern::Value(val),
    };

    match expected_nan_pattern {
        NanPattern::CanonicalNan => {
            let canon_nan = 0x7fc0_0000;
            if (actual_bits & 0x7fff_ffff) == canon_nan {
                Ok(())
            } else {
                Err(AssertEqError {
                    left: actual_bits.to_string(),
                    right: canon_nan.to_string(),
                })
            }
        }
        NanPattern::ArithmeticNan => {
            const AF32_NAN: u32 = 0x7f80_0000;
            let is_nan = actual_bits & AF32_NAN == AF32_NAN;
            const AF32_PAYLOAD_MSB: u32 = 0x0040_0000;
            let is_msb_set = actual_bits & AF32_PAYLOAD_MSB == AF32_PAYLOAD_MSB;
            if is_nan && is_msb_set {
                Ok(())
            } else {
                Err(AssertEqError {
                    left: actual_bits.to_string(),
                    right: AF32_NAN.to_string(),
                })
            }
        }
        NanPattern::Value(val) => {
            if actual_bits == val {
                Ok(())
            } else {
                Err(AssertEqError {
                    left: actual_bits.to_string(),
                    right: val.to_string(),
                })
            }
        }
    }
}

fn match_f64(actual: F64, expected: F64) -> Result<(), AssertEqError> {
    use wast::core::NanPattern;

    let actual_bits = actual.to_bits();

    let expected_nan_pattern: NanPattern<u64> = match expected.to_bits() {
        0x7ff8_0000_0000_0000 => NanPattern::CanonicalNan,
        // Inf and ArithmeticNan might overlap, and since we cast to our Value type and then back to NanPattern we need the distinction
        0x7ff0_0000_0000_0001 => NanPattern::ArithmeticNan,
        val => NanPattern::Value(val),
    };

    match expected_nan_pattern {
        NanPattern::CanonicalNan => {
            let canon_nan = 0x7ff8_0000_0000_0000;
            if (actual_bits & 0x7fff_ffff_ffff_ffff) == canon_nan {
                Ok(())
            } else {
                Err(AssertEqError {
                    left: actual_bits.to_string(),
                    right: canon_nan.to_string(),
                })
            }
        }
        NanPattern::ArithmeticNan => {
            const AF64_NAN: u64 = 0x7ff0_0000_0000_0000;
            let is_nan = actual_bits & AF64_NAN == AF64_NAN;
            const AF64_PAYLOAD_MSB: u64 = 0x0008_0000_0000_0000;
            let is_msb_set = actual_bits & AF64_PAYLOAD_MSB == AF64_PAYLOAD_MSB;
            if is_nan && is_msb_set {
                Ok(())
            } else {
                Err(AssertEqError {
                    left: actual_bits.to_string(),
                    right: AF64_NAN.to_string(),
                })
            }
        }
        NanPattern::Value(val) => {
            if actual_bits == val {
                Ok(())
            } else {
                Err(AssertEqError {
                    left: actual_bits.to_string(),
                    right: val.to_string(),
                })
            }
        }
    }
}

impl Error for AssertEqError {}
impl std::fmt::Display for AssertEqError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "assert_eq failed: left: {}, right: {}",
            self.left, self.right
        )
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PanicError {
    message: String,
}

impl PanicError {
    pub fn new(message: &str) -> Self {
        PanicError {
            message: message.to_string(),
        }
    }

    pub fn from_panic(panic: Box<dyn std::any::Any + Send>) -> Self {
        if let Ok(msg) = panic.downcast::<&str>() {
            PanicError::new(&msg)
        } else {
            PanicError::new("Unknown panic")
        }
    }

    pub fn from_panic_boxed(panic: Box<dyn std::any::Any + Send>) -> Box<dyn Error> {
        Box::new(Self::from_panic(panic))
    }
}

impl Error for PanicError {}
impl std::fmt::Display for PanicError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Panic: {}", self.message)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WasmInterpreterError(pub wasm::Error);

impl WasmInterpreterError {
    pub fn new_boxed(error: wasm::Error) -> Box<dyn Error> {
        Box::new(Self(error))
    }
}

impl Error for WasmInterpreterError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.0 {
            wasm::Error::MalformedUtf8String(inner) => Some(inner),
            _ => None,
        }
    }
}

impl std::fmt::Display for WasmInterpreterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GenericError(String);

impl GenericError {
    pub fn new(message: &str) -> Self {
        GenericError(message.to_string())
    }

    pub fn new_boxed(message: &str) -> Box<dyn Error> {
        Box::new(Self::new(message))
    }
}

impl Error for GenericError {}
impl std::fmt::Display for GenericError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
