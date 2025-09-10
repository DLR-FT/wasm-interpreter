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
    value::{ExternAddr, FuncAddr, Ref, F32, F64},
    Value,
};
use wast::core::{AbstractHeapType, HeapType, NanPattern, WastRetCore};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AssertEqError {
    left: String,
    right: String,
}

impl AssertEqError {
    pub fn assert_eq(actual: Vec<Value>, expected: Vec<WastRetCore>) -> Result<(), Self> {
        if actual.len() != expected.len() {
            return Err(AssertEqError {
                left: format!("Arr<len: {}>", actual.len()),
                right: format!("Arr<len: {}>", expected.len()),
            });
        }

        actual
            .into_iter()
            .zip(expected)
            .try_for_each(|(actual, expected)| {
                let values_equal = match (actual, &expected) {
                    (Value::I32(actual), WastRetCore::I32(expected)) => actual == *expected as u32,
                    (Value::I64(actual), WastRetCore::I64(expected)) => actual == *expected as u64,
                    (Value::F32(actual), WastRetCore::F32(expected)) => {
                        match_f32(actual, *expected)
                    }
                    (Value::F64(actual), WastRetCore::F64(expected)) => {
                        match_f64(actual, *expected)
                    }
                    (_, WastRetCore::V128(_expected)) => {
                        todo!("implement vector types")
                    }
                    (Value::Ref(Ref::Extern(actual)), WastRetCore::RefExtern(expected)) => {
                        actual.addr == expected.map(|x| x as usize)
                    }
                    (Value::Ref(Ref::Func(_actual)), WastRetCore::RefFunc(_expected)) => {
                        todo!("implement funcref types")
                    }
                    (
                        Value::Ref(Ref::Extern(ExternAddr { addr: None })),
                        WastRetCore::RefNull(Some(HeapType::Abstract {
                            ty: AbstractHeapType::Extern,
                            ..
                        })),
                    ) => true,
                    (
                        Value::Ref(Ref::Func(FuncAddr { addr: None })),
                        WastRetCore::RefNull(Some(HeapType::Abstract {
                            ty: AbstractHeapType::Func,
                            ..
                        })),
                    ) => true,
                    _ => false,
                };

                values_equal.then_some(()).ok_or_else(|| AssertEqError {
                    left: format!("{actual:?}"),
                    right: format!("{expected:?}"),
                })
            })
    }
}

fn match_f32(actual: F32, expected: NanPattern<wast::token::F32>) -> bool {
    let actual_bits = actual.to_bits();

    match expected {
        NanPattern::CanonicalNan => {
            let canon_nan = 0x7fc0_0000;
            (actual_bits & 0x7fff_ffff) == canon_nan
        }
        NanPattern::ArithmeticNan => {
            const AF32_NAN: u32 = 0x7f80_0000;
            let is_nan = actual_bits & AF32_NAN == AF32_NAN;
            const AF32_PAYLOAD_MSB: u32 = 0x0040_0000;
            let is_msb_set = actual_bits & AF32_PAYLOAD_MSB == AF32_PAYLOAD_MSB;
            is_nan && is_msb_set
        }
        NanPattern::Value(val) => actual_bits == val.bits,
    }
}

fn match_f64(actual: F64, expected: NanPattern<wast::token::F64>) -> bool {
    let actual_bits = actual.to_bits();

    match expected {
        NanPattern::CanonicalNan => {
            let canon_nan = 0x7ff8_0000_0000_0000;
            (actual_bits & 0x7fff_ffff_ffff_ffff) == canon_nan
        }
        NanPattern::ArithmeticNan => {
            const AF64_NAN: u64 = 0x7ff0_0000_0000_0000;
            let is_nan = actual_bits & AF64_NAN == AF64_NAN;
            const AF64_PAYLOAD_MSB: u64 = 0x0008_0000_0000_0000;
            let is_msb_set = actual_bits & AF64_PAYLOAD_MSB == AF64_PAYLOAD_MSB;
            is_nan && is_msb_set
        }
        NanPattern::Value(val) => actual_bits == val.bits,
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
