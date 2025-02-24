/*
# This file incorporates code from the WebAssembly testsuite, originally
# available at https://github.com/WebAssembly/testsuite.
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
use core::{f32, f64};

use hexf::{hexf32, hexf64};
use wasm::{validate, RuntimeError, RuntimeInstance};

const WAT: &str = r#"
      (module
      (func (export "{{0}}") (param $x {{1}}) (result {{2}})
          local.get $x
          {{0}})
    )
"#;

#[should_panic(expected = "validation failed: InvalidValidationStackValType(Some(NumType(I32)))")]
#[test_log::test]
pub fn i32_wrap_i64_let_it_die() {
    let wat = String::from(WAT)
        .replace("{{0}}", "i32.wrap_i64")
        .replace("{{1}}", "i32")
        .replace("{{2}}", "i32");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        -1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1)
            .unwrap()
    );
}

/// A function to test the i32.wrap_i64 implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/7570678ade1244ae69c9fefc990f4534c63ffaec/conversions.wast#L51)
#[test_log::test]
pub fn i32_wrap_i64() {
    let wat = String::from(WAT)
        .replace("{{0}}", "i32.wrap_i64")
        .replace("{{1}}", "i64")
        .replace("{{2}}", "i32");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        -1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1_i64)
            .unwrap()
    );
    assert_eq!(
        -100000,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -100000_i64)
            .unwrap()
    );
    assert_eq!(
        0x80000000_u32 as i32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0x80000000_i64
            )
            .unwrap()
    );
    assert_eq!(
        0x7fffffff,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0xffffffff7fffffff_u64 as i64
            )
            .unwrap()
    );
    assert_eq!(
        0x00000000,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0xffffffff00000000_u64 as i64
            )
            .unwrap()
    );
    assert_eq!(
        0xffffffff_u32 as i32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0xfffffffeffffffff_u64 as i64
            )
            .unwrap()
    );
    assert_eq!(
        0x00000001,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0xffffffff00000001_u64 as i64
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0_i64)
            .unwrap()
    );
    assert_eq!(
        0x9abcdef0_u32 as i32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                1311768467463790320_i64
            )
            .unwrap()
    );
    assert_eq!(
        0xffffffff_u32 as i32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0x00000000ffffffff_i64
            )
            .unwrap()
    );
    assert_eq!(
        0x00000000,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0x0000000100000000_i64
            )
            .unwrap()
    );
    assert_eq!(
        0x00000001,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0x0000000100000001_i64
            )
            .unwrap()
    );
}

#[should_panic(expected = "validation failed: InvalidValidationStackValType(Some(NumType(I32)))")]
#[test_log::test]
pub fn i32_trunc_f32_s_let_it_die() {
    let wat = String::from(WAT)
        .replace("{{0}}", "i32.trunc_f32_s")
        .replace("{{1}}", "i32")
        .replace("{{2}}", "i32");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        -1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1)
            .unwrap()
    );
}

/// A function to test the i32.trunc_f32_s implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/7570678ade1244ae69c9fefc990f4534c63ffaec/conversions.wast#L64)
#[test_log::test]
pub fn i32_trunc_f32_s() {
    let wat = String::from(WAT)
        .replace("{{0}}", "i32.trunc_f32_s")
        .replace("{{1}}", "f32")
        .replace("{{2}}", "i32");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0.0_f32)
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -0.0_f32)
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf32!("0x1p-149")
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf32!("-0x1p-149")
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1.0_f32)
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf32!("0x1.19999ap+0")
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1.5_f32)
            .unwrap()
    );
    assert_eq!(
        -1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1.0_f32)
            .unwrap()
    );
    assert_eq!(
        -1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf32!("-0x1.19999ap+0")
            )
            .unwrap()
    );
    assert_eq!(
        -1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1.5_f32)
            .unwrap()
    );
    assert_eq!(
        -1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1.9_f32)
            .unwrap()
    );
    assert_eq!(
        -2,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -2.0_f32)
            .unwrap()
    );
    assert_eq!(
        2147483520,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                2147483520.0_f32
            )
            .unwrap()
    );
    assert_eq!(
        2147483648_u32 as i32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                -2147483648.0_f32
            )
            .unwrap()
    );
    assert_eq!(
        RuntimeError::UnrepresentableResult,
        instance
            .invoke::<f32, i32>(
                &instance.get_function_by_index(0, 0).unwrap(),
                2147483648.0_f32
            )
            .err()
            .unwrap()
    );
    assert_eq!(
        RuntimeError::UnrepresentableResult,
        instance
            .invoke::<f32, i32>(
                &instance.get_function_by_index(0, 0).unwrap(),
                -2147483904.0_f32
            )
            .err()
            .unwrap()
    );
    assert_eq!(
        RuntimeError::UnrepresentableResult,
        instance
            .invoke::<f32, i32>(
                &instance.get_function_by_index(0, 0).unwrap(),
                f32::INFINITY
            )
            .err()
            .unwrap()
    );
    assert_eq!(
        RuntimeError::UnrepresentableResult,
        instance
            .invoke::<f32, i32>(
                &instance.get_function_by_index(0, 0).unwrap(),
                f32::NEG_INFINITY
            )
            .err()
            .unwrap()
    );
    assert_eq!(
        RuntimeError::BadConversionToInteger,
        instance
            .invoke::<f32, i32>(&instance.get_function_by_index(0, 0).unwrap(), f32::NAN)
            .err()
            .unwrap()
    );
    assert_eq!(
        RuntimeError::BadConversionToInteger,
        instance
            .invoke::<f32, i32>(&instance.get_function_by_index(0, 0).unwrap(), -f32::NAN)
            .err()
            .unwrap()
    );
}

/// A function to test the i32.trunc_f32_u implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/7570678ade1244ae69c9fefc990f4534c63ffaec/conversions.wast#L87C1-L107C99)
#[test_log::test]
pub fn i32_trunc_f32_u() {
    let wat = String::from(WAT)
        .replace("{{0}}", "i32.trunc_f32_u")
        .replace("{{1}}", "f32")
        .replace("{{2}}", "i32");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0.0_f32)
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -0.0_f32)
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf32!("0x1p-149")
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf32!("-0x1p-149")
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1.0_f32)
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf32!("0x1.19999ap+0")
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1.5_f32)
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1.9_f32)
            .unwrap()
    );
    assert_eq!(
        2,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 2.0_f32)
            .unwrap()
    );
    assert_eq!(
        -2147483648,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                2147483648_f32
            )
            .unwrap()
    );
    assert_eq!(
        -256,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                4294967040.0_f32
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf32!("-0x1.ccccccp-1")
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf32!("-0x1.fffffep-1")
            )
            .unwrap()
    );
    assert_eq!(
        RuntimeError::UnrepresentableResult,
        instance
            .invoke::<f32, i32>(
                &instance.get_function_by_index(0, 0).unwrap(),
                4294967296.0_f32
            )
            .err()
            .unwrap()
    );
    assert_eq!(
        RuntimeError::UnrepresentableResult,
        instance
            .invoke::<f32, i32>(&instance.get_function_by_index(0, 0).unwrap(), -1.0)
            .err()
            .unwrap()
    );
    assert_eq!(
        RuntimeError::UnrepresentableResult,
        instance
            .invoke::<f32, i32>(
                &instance.get_function_by_index(0, 0).unwrap(),
                f32::INFINITY
            )
            .err()
            .unwrap()
    );
    assert_eq!(
        RuntimeError::UnrepresentableResult,
        instance
            .invoke::<f32, i32>(
                &instance.get_function_by_index(0, 0).unwrap(),
                f32::NEG_INFINITY
            )
            .err()
            .unwrap()
    );
    assert_eq!(
        RuntimeError::BadConversionToInteger,
        instance
            .invoke::<f32, i32>(&instance.get_function_by_index(0, 0).unwrap(), f32::NAN)
            .err()
            .unwrap()
    );
    assert_eq!(
        RuntimeError::BadConversionToInteger,
        instance
            .invoke::<f32, i32>(&instance.get_function_by_index(0, 0).unwrap(), -f32::NAN)
            .err()
            .unwrap()
    );
}

#[should_panic(expected = "validation failed: InvalidValidationStackValType(Some(NumType(I32)))")]
#[test_log::test]
pub fn i32_trunc_f64_s_let_it_die() {
    let wat = String::from(WAT)
        .replace("{{0}}", "i32.trunc_f64_s")
        .replace("{{1}}", "i32")
        .replace("{{2}}", "i32");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        -1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1)
            .unwrap()
    );
}

/// A function to test the i32.trunc_f64_s implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/7570678ade1244ae69c9fefc990f4534c63ffaec/conversions.wast#L109)
#[test_log::test]
pub fn i32_trunc_f64_s() {
    let wat = String::from(WAT)
        .replace("{{0}}", "i32.trunc_f64_s")
        .replace("{{1}}", "f64")
        .replace("{{2}}", "i32");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0.0_f64)
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -0.0_f64)
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("0x0.0000000000001p-1022")
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("-0x0.0000000000001p-1022")
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1.0_f64)
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("0x1.199999999999ap+0")
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1.5_f64)
            .unwrap()
    );
    assert_eq!(
        -1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1.0_f64)
            .unwrap()
    );
    assert_eq!(
        -1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("-0x1.199999999999ap+0")
            )
            .unwrap()
    );
    assert_eq!(
        -1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1.5_f64)
            .unwrap()
    );
    assert_eq!(
        -1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1.9_f64)
            .unwrap()
    );
    assert_eq!(
        -2,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -2.0_f64)
            .unwrap()
    );
    assert_eq!(
        2147483647,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                2147483647.0_f64
            )
            .unwrap()
    );
    assert_eq!(
        -2147483648,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                -2147483648.0_f64
            )
            .unwrap()
    );
    assert_eq!(
        -2147483648,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                -2147483648.9_f64
            )
            .unwrap()
    );
    assert_eq!(
        -2147483648,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                -2147483648.9_f64
            )
            .unwrap()
    );
    assert_eq!(
        RuntimeError::UnrepresentableResult,
        instance
            .invoke::<f64, i32>(&instance.get_function_by_index(0, 0).unwrap(), 2147483648.0)
            .err()
            .unwrap()
    );
    assert_eq!(
        RuntimeError::UnrepresentableResult,
        instance
            .invoke::<f64, i32>(
                &instance.get_function_by_index(0, 0).unwrap(),
                -2147483649.0
            )
            .err()
            .unwrap()
    );
    assert_eq!(
        RuntimeError::UnrepresentableResult,
        instance
            .invoke::<f64, i32>(
                &instance.get_function_by_index(0, 0).unwrap(),
                f64::INFINITY
            )
            .err()
            .unwrap()
    );
    assert_eq!(
        RuntimeError::UnrepresentableResult,
        instance
            .invoke::<f64, i32>(
                &instance.get_function_by_index(0, 0).unwrap(),
                f64::NEG_INFINITY
            )
            .err()
            .unwrap()
    );
    assert_eq!(
        RuntimeError::BadConversionToInteger,
        instance
            .invoke::<f64, i32>(&instance.get_function_by_index(0, 0).unwrap(), f64::NAN)
            .err()
            .unwrap()
    );
    assert_eq!(
        RuntimeError::BadConversionToInteger,
        instance
            .invoke::<f64, i32>(&instance.get_function_by_index(0, 0).unwrap(), -f64::NAN)
            .err()
            .unwrap()
    );
}

/// A function to test the i32.trunc_f64_u implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/7570678ade1244ae69c9fefc990f4534c63ffaec/conversions.wast#L134)
#[test_log::test]
pub fn i32_trunc_f64_u() {
    let wat = String::from(WAT)
        .replace("{{0}}", "i32.trunc_f64_u")
        .replace("{{1}}", "f64")
        .replace("{{2}}", "i32");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0.0_f64)
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -0.0_f64)
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("0x0.0000000000001p-1022")
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("-0x0.0000000000001p-1022")
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1.0_f64)
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("0x1.199999999999ap+0")
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1.5_f64)
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1.9_f64)
            .unwrap()
    );
    assert_eq!(
        2,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 2.0_f64)
            .unwrap()
    );
    assert_eq!(
        -2147483648,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                2147483648_f64
            )
            .unwrap()
    );
    assert_eq!(
        -1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                4294967295.0_f64
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("-0x1.ccccccccccccdp-1")
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("-0x1.fffffffffffffp-1")
            )
            .unwrap()
    );
    assert_eq!(
        100000000,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1e8_f64)
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -0.9)
            .unwrap()
    );
    assert_eq!(
        4294967295_u32 as i32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                4294967295.9_f64
            )
            .unwrap()
    );
    assert_eq!(
        RuntimeError::UnrepresentableResult,
        instance
            .invoke::<f64, i32>(&instance.get_function_by_index(0, 0).unwrap(), 4294967296.0)
            .err()
            .unwrap()
    );
    assert_eq!(
        RuntimeError::UnrepresentableResult,
        instance
            .invoke::<f64, i32>(&instance.get_function_by_index(0, 0).unwrap(), -1.0)
            .err()
            .unwrap()
    );
    assert_eq!(
        RuntimeError::UnrepresentableResult,
        instance
            .invoke::<f64, i32>(&instance.get_function_by_index(0, 0).unwrap(), 1e16)
            .err()
            .unwrap()
    );
    assert_eq!(
        RuntimeError::UnrepresentableResult,
        instance
            .invoke::<f64, i32>(&instance.get_function_by_index(0, 0).unwrap(), 1e30)
            .err()
            .unwrap()
    );
    assert_eq!(
        RuntimeError::UnrepresentableResult,
        instance
            .invoke::<f64, i32>(
                &instance.get_function_by_index(0, 0).unwrap(),
                9223372036854775808_f64
            )
            .err()
            .unwrap()
    );
    assert_eq!(
        RuntimeError::UnrepresentableResult,
        instance
            .invoke::<f64, i32>(
                &instance.get_function_by_index(0, 0).unwrap(),
                f64::INFINITY
            )
            .err()
            .unwrap()
    );
    assert_eq!(
        RuntimeError::UnrepresentableResult,
        instance
            .invoke::<f64, i32>(
                &instance.get_function_by_index(0, 0).unwrap(),
                f64::NEG_INFINITY
            )
            .err()
            .unwrap()
    );
    assert_eq!(
        RuntimeError::BadConversionToInteger,
        instance
            .invoke::<f64, i32>(&instance.get_function_by_index(0, 0).unwrap(), f64::NAN)
            .err()
            .unwrap()
    );
    assert_eq!(
        RuntimeError::BadConversionToInteger,
        instance
            .invoke::<f64, i32>(&instance.get_function_by_index(0, 0).unwrap(), -f64::NAN)
            .err()
            .unwrap()
    );
}

#[should_panic(expected = "validation failed: InvalidValidationStackValType(Some(NumType(I64)))")]
#[test_log::test]
pub fn i64_extend_i32_s_let_it_die() {
    let wat = String::from(WAT)
        .replace("{{0}}", "i64.extend_i32_u")
        .replace("{{1}}", "i64")
        .replace("{{2}}", "i64");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        -1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1)
            .unwrap()
    );
}

/// A function to test the i64.extend_i32_s implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/7570678ade1244ae69c9fefc990f4534c63ffaec/conversions.wast#L37)
#[test_log::test]
pub fn i64_extend_i32_s() {
    let wat = String::from(WAT)
        .replace("{{0}}", "i64.extend_i32_s")
        .replace("{{1}}", "i32")
        .replace("{{2}}", "i64");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        0_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0)
            .unwrap()
    );
    assert_eq!(
        10000_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 10000)
            .unwrap()
    );
    assert_eq!(
        -10000_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -10000)
            .unwrap()
    );
    assert_eq!(
        -1_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1)
            .unwrap()
    );
    assert_eq!(
        0x000000007fffffff_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0x7fffffff)
            .unwrap()
    );
    assert_eq!(
        0xffffffff80000000_u64 as i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0x80000000_u32 as i32
            )
            .unwrap()
    );
}

/// A function to test the i64.extend_i32_u implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/7570678ade1244ae69c9fefc990f4534c63ffaec/conversions.wast#L44C1-L49C98)
#[test_log::test]
pub fn i64_extend_i32_u() {
    let wat = String::from(WAT)
        .replace("{{0}}", "i64.extend_i32_u")
        .replace("{{1}}", "i32")
        .replace("{{2}}", "i64");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        0_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0)
            .unwrap()
    );
    assert_eq!(
        10000_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 10000)
            .unwrap()
    );
    assert_eq!(
        0x00000000ffffd8f0_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -10000)
            .unwrap()
    );
    assert_eq!(
        0xffffffff_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1)
            .unwrap()
    );
    assert_eq!(
        0x000000007fffffff_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0x7fffffff)
            .unwrap()
    );
    assert_eq!(
        0x0000000080000000_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0x80000000_u32 as i32
            )
            .unwrap()
    );
}

#[should_panic(expected = "validation failed: InvalidValidationStackValType(Some(NumType(I64)))")]
#[test_log::test]
pub fn i64_trunc_f32_s_let_it_die() {
    let wat = String::from(WAT)
        .replace("{{0}}", "i64.trunc_f32_s")
        .replace("{{1}}", "i64")
        .replace("{{2}}", "i64");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        -1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1)
            .unwrap()
    );
}

/// A function to test the i64.trunc_f32_s implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/7570678ade1244ae69c9fefc990f4534c63ffaec/conversions.wast#L162)
#[test_log::test]
pub fn i64_trunc_f32_s() {
    let wat = String::from(WAT)
        .replace("{{0}}", "i64.trunc_f32_s")
        .replace("{{1}}", "f32")
        .replace("{{2}}", "i64");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        0_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0.0_f32)
            .unwrap()
    );
    assert_eq!(
        0_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -0.0_f32)
            .unwrap()
    );
    assert_eq!(
        0_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf32!("0x1p-149")
            )
            .unwrap()
    );
    assert_eq!(
        0_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf32!("-0x1p-149")
            )
            .unwrap()
    );
    assert_eq!(
        1_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1.0_f32)
            .unwrap()
    );
    assert_eq!(
        1_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf32!("0x1.19999ap+0")
            )
            .unwrap()
    );
    assert_eq!(
        1_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1.5_f32)
            .unwrap()
    );
    assert_eq!(
        -1_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1.0_f32)
            .unwrap()
    );
    assert_eq!(
        -1_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf32!("-0x1.19999ap+0")
            )
            .unwrap()
    );
    assert_eq!(
        -1_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1.5_f32)
            .unwrap()
    );
    assert_eq!(
        -1_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1.9_f32)
            .unwrap()
    );
    assert_eq!(
        -2_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -2.0_f32)
            .unwrap()
    );
    assert_eq!(
        4294967296_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                4294967296_f32
            )
            .unwrap()
    );
    assert_eq!(
        -4294967296_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                -4294967296_f32
            )
            .unwrap()
    );
    assert_eq!(
        9223371487098961920_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                9223371487098961920.0_f32
            )
            .unwrap()
    );
    assert_eq!(
        -9223372036854775808_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                -9223372036854775808.0_f32
            )
            .unwrap()
    );
    assert_eq!(
        RuntimeError::UnrepresentableResult,
        instance
            .invoke::<f32, i64>(
                &instance.get_function_by_index(0, 0).unwrap(),
                9223372036854775808.0_f32
            )
            .err()
            .unwrap()
    );
    assert_eq!(
        RuntimeError::UnrepresentableResult,
        instance
            .invoke::<f32, i64>(
                &instance.get_function_by_index(0, 0).unwrap(),
                -9223373136366403584.0_f32
            )
            .err()
            .unwrap()
    );
    assert_eq!(
        RuntimeError::UnrepresentableResult,
        instance
            .invoke::<f32, i64>(
                &instance.get_function_by_index(0, 0).unwrap(),
                f32::INFINITY
            )
            .err()
            .unwrap()
    );
    assert_eq!(
        RuntimeError::UnrepresentableResult,
        instance
            .invoke::<f32, i64>(
                &instance.get_function_by_index(0, 0).unwrap(),
                f32::NEG_INFINITY
            )
            .err()
            .unwrap()
    );
    assert_eq!(
        RuntimeError::BadConversionToInteger,
        instance
            .invoke::<f32, i64>(&instance.get_function_by_index(0, 0).unwrap(), f32::NAN)
            .err()
            .unwrap()
    );
    assert_eq!(
        RuntimeError::BadConversionToInteger,
        instance
            .invoke::<f32, i64>(&instance.get_function_by_index(0, 0).unwrap(), -f32::NAN)
            .err()
            .unwrap()
    );
}

/// A function to test the i64.trunc_f32_u implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/7570678ade1244ae69c9fefc990f4534c63ffaec/conversions.wast#L187)
#[test_log::test]
pub fn i64_trunc_f32_u() {
    let wat = String::from(WAT)
        .replace("{{0}}", "i64.trunc_f32_u")
        .replace("{{1}}", "f32")
        .replace("{{2}}", "i64");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        0_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0.0_f32)
            .unwrap()
    );
    assert_eq!(
        0_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -0.0_f32)
            .unwrap()
    );
    assert_eq!(
        0_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf32!("0x1p-149")
            )
            .unwrap()
    );
    assert_eq!(
        0_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf32!("-0x1p-149")
            )
            .unwrap()
    );
    assert_eq!(
        1_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1.0_f32)
            .unwrap()
    );
    assert_eq!(
        1_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf32!("0x1.19999ap+0")
            )
            .unwrap()
    );
    assert_eq!(
        1_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1.5_f32)
            .unwrap()
    );
    assert_eq!(
        4294967296_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                4294967296_f32
            )
            .unwrap()
    );
    assert_eq!(
        -1099511627776_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                18446742974197923840.0_f32
            )
            .unwrap()
    );
    assert_eq!(
        1_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf32!("0x1.19999ap+0")
            )
            .unwrap()
    );
    assert_eq!(
        0_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf32!("-0x1.fffffep-1")
            )
            .unwrap()
    );
    assert_eq!(
        RuntimeError::UnrepresentableResult,
        instance
            .invoke::<f32, i64>(
                &instance.get_function_by_index(0, 0).unwrap(),
                18446744073709551616.0_f32
            )
            .err()
            .unwrap()
    );
    assert_eq!(
        RuntimeError::UnrepresentableResult,
        instance
            .invoke::<f32, i64>(&instance.get_function_by_index(0, 0).unwrap(), -1.0_f32)
            .err()
            .unwrap()
    );
    assert_eq!(
        RuntimeError::UnrepresentableResult,
        instance
            .invoke::<f32, i64>(
                &instance.get_function_by_index(0, 0).unwrap(),
                f32::INFINITY
            )
            .err()
            .unwrap()
    );
    assert_eq!(
        RuntimeError::UnrepresentableResult,
        instance
            .invoke::<f32, i64>(
                &instance.get_function_by_index(0, 0).unwrap(),
                f32::NEG_INFINITY
            )
            .err()
            .unwrap()
    );
    assert_eq!(
        RuntimeError::BadConversionToInteger,
        instance
            .invoke::<f32, i64>(&instance.get_function_by_index(0, 0).unwrap(), f32::NAN)
            .err()
            .unwrap()
    );
    assert_eq!(
        RuntimeError::BadConversionToInteger,
        instance
            .invoke::<f32, i64>(&instance.get_function_by_index(0, 0).unwrap(), -f32::NAN)
            .err()
            .unwrap()
    );
}

#[should_panic(expected = "validation failed: InvalidValidationStackValType(Some(NumType(I64)))")]
#[test_log::test]
pub fn i64_trunc_f64_s_let_it_die() {
    let wat = String::from(WAT)
        .replace("{{0}}", "i64.trunc_f64_s")
        .replace("{{1}}", "i64")
        .replace("{{2}}", "i64");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        -1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1)
            .unwrap()
    );
}

/// A function to test the i64.trunc_f64_s implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/7570678ade1244ae69c9fefc990f4534c63ffaec/conversions.wast#L207)
#[test_log::test]
pub fn i64_trunc_f64_s() {
    let wat = String::from(WAT)
        .replace("{{0}}", "i64.trunc_f64_s")
        .replace("{{1}}", "f64")
        .replace("{{2}}", "i64");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        0_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0.0_f64)
            .unwrap()
    );
    assert_eq!(
        0_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -0.0_f64)
            .unwrap()
    );
    assert_eq!(
        0_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("0x0.0000000000001p-1022")
            )
            .unwrap()
    );
    assert_eq!(
        0_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("-0x0.0000000000001p-1022")
            )
            .unwrap()
    );
    assert_eq!(
        1_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1.0_f64)
            .unwrap()
    );
    assert_eq!(
        1_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("0x1.199999999999ap+0")
            )
            .unwrap()
    );
    assert_eq!(
        1_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1.5_f64)
            .unwrap()
    );
    assert_eq!(
        -1_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1.0_f64)
            .unwrap()
    );
    assert_eq!(
        -1_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("-0x1.199999999999ap+0")
            )
            .unwrap()
    );
    assert_eq!(
        -1_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1.5_f64)
            .unwrap()
    );
    assert_eq!(
        -1_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1.9_f64)
            .unwrap()
    );
    assert_eq!(
        -2_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -2.0_f64)
            .unwrap()
    );
    assert_eq!(
        4294967296_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                4294967296_f64
            )
            .unwrap()
    );
    assert_eq!(
        -4294967296_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                -4294967296_f64
            )
            .unwrap()
    );
    assert_eq!(
        9223372036854774784_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                9223372036854774784.0_f64
            )
            .unwrap()
    );
    assert_eq!(
        -9223372036854775808_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                -9223372036854775808.0_f64
            )
            .unwrap()
    );
    assert_eq!(
        RuntimeError::UnrepresentableResult,
        instance
            .invoke::<f64, i64>(
                &instance.get_function_by_index(0, 0).unwrap(),
                9223372036854775808.0_f64
            )
            .err()
            .unwrap()
    );
    assert_eq!(
        RuntimeError::UnrepresentableResult,
        instance
            .invoke::<f64, i64>(
                &instance.get_function_by_index(0, 0).unwrap(),
                -9223372036854777856.0_f64
            )
            .err()
            .unwrap()
    );
    assert_eq!(
        RuntimeError::UnrepresentableResult,
        instance
            .invoke::<f64, i64>(
                &instance.get_function_by_index(0, 0).unwrap(),
                f64::INFINITY
            )
            .err()
            .unwrap()
    );
    assert_eq!(
        RuntimeError::UnrepresentableResult,
        instance
            .invoke::<f64, i64>(
                &instance.get_function_by_index(0, 0).unwrap(),
                f64::NEG_INFINITY
            )
            .err()
            .unwrap()
    );
    assert_eq!(
        RuntimeError::BadConversionToInteger,
        instance
            .invoke::<f64, i64>(&instance.get_function_by_index(0, 0).unwrap(), f64::NAN)
            .err()
            .unwrap()
    );
    assert_eq!(
        RuntimeError::BadConversionToInteger,
        instance
            .invoke::<f64, i64>(&instance.get_function_by_index(0, 0).unwrap(), -f64::NAN)
            .err()
            .unwrap()
    );
}

/// A function to test the i64.trunc_f64_u implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/7570678ade1244ae69c9fefc990f4534c63ffaec/conversions.wast#L232)
#[test_log::test]
pub fn i64_trunc_f64_u() {
    let wat = String::from(WAT)
        .replace("{{0}}", "i64.trunc_f64_u")
        .replace("{{1}}", "f64")
        .replace("{{2}}", "i64");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        0_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0.0_f64)
            .unwrap()
    );
    assert_eq!(
        0_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -0.0_f64)
            .unwrap()
    );
    assert_eq!(
        0_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("0x0.0000000000001p-1022")
            )
            .unwrap()
    );
    assert_eq!(
        0_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("-0x0.0000000000001p-1022")
            )
            .unwrap()
    );
    assert_eq!(
        1_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1.0_f64)
            .unwrap()
    );
    assert_eq!(
        1_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("0x1.199999999999ap+0")
            )
            .unwrap()
    );
    assert_eq!(
        1_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1.5_f64)
            .unwrap()
    );
    assert_eq!(
        0xffffffff_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                4294967295_f64
            )
            .unwrap()
    );
    assert_eq!(
        0x100000000_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                4294967296_f64
            )
            .unwrap()
    );
    assert_eq!(
        -2048_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                18446744073709549568.0_f64
            )
            .unwrap()
    );
    assert_eq!(
        0_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("-0x1.ccccccccccccdp-1")
            )
            .unwrap()
    );
    assert_eq!(
        0_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("-0x1.fffffffffffffp-1")
            )
            .unwrap()
    );
    assert_eq!(
        100000000_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1e8_f64)
            .unwrap()
    );
    assert_eq!(
        10000000000000000_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1e16_f64)
            .unwrap()
    );
    assert_eq!(
        -9223372036854775808_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                9223372036854775808_f64
            )
            .unwrap()
    );
    assert_eq!(
        RuntimeError::UnrepresentableResult,
        instance
            .invoke::<f64, i64>(
                &instance.get_function_by_index(0, 0).unwrap(),
                18446744073709551616.0_f64
            )
            .err()
            .unwrap()
    );
    assert_eq!(
        RuntimeError::UnrepresentableResult,
        instance
            .invoke::<f64, i64>(&instance.get_function_by_index(0, 0).unwrap(), -1_f64)
            .err()
            .unwrap()
    );
    assert_eq!(
        RuntimeError::UnrepresentableResult,
        instance
            .invoke::<f64, i64>(
                &instance.get_function_by_index(0, 0).unwrap(),
                f64::INFINITY
            )
            .err()
            .unwrap()
    );
    assert_eq!(
        RuntimeError::UnrepresentableResult,
        instance
            .invoke::<f64, i64>(
                &instance.get_function_by_index(0, 0).unwrap(),
                f64::NEG_INFINITY
            )
            .err()
            .unwrap()
    );
    assert_eq!(
        RuntimeError::BadConversionToInteger,
        instance
            .invoke::<f64, i64>(&instance.get_function_by_index(0, 0).unwrap(), f64::NAN)
            .err()
            .unwrap()
    );
    assert_eq!(
        RuntimeError::BadConversionToInteger,
        instance
            .invoke::<f64, i64>(&instance.get_function_by_index(0, 0).unwrap(), -f64::NAN)
            .err()
            .unwrap()
    );
}

#[should_panic(expected = "validation failed: InvalidValidationStackValType(Some(NumType(I64)))")]
#[test_log::test]
pub fn f32_convert_i32_s_let_it_die() {
    let wat = String::from(WAT)
        .replace("{{0}}", "f32.convert_i32_s")
        .replace("{{1}}", "i64")
        .replace("{{2}}", "f32");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        -1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1)
            .unwrap()
    );
}

/// A function to test the f32.convert_i32_s implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/7570678ade1244ae69c9fefc990f4534c63ffaec/conversions.wast#L256)
#[test_log::test]
pub fn f32_convert_i32_s() {
    let wat = String::from(WAT)
        .replace("{{0}}", "f32.convert_i32_s")
        .replace("{{1}}", "i32")
        .replace("{{2}}", "f32");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        1.0_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1)
            .unwrap()
    );
    assert_eq!(
        -1.0_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1)
            .unwrap()
    );
    assert_eq!(
        0.0_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0)
            .unwrap()
    );
    assert_eq!(
        2147483648_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 2147483647)
            .unwrap()
    );
    assert_eq!(
        -2147483648_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -2147483648)
            .unwrap()
    );
    assert_eq!(
        hexf32!("0x1.26580cp+30"),
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1234567890)
            .unwrap()
    );
}

/// A function to test the f32.convert_i32_s implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/7570678ade1244ae69c9fefc990f4534c63ffaec/conversions.wast#L495)
#[test_log::test]
pub fn f32_convert_i32_u() {
    let wat = String::from(WAT)
        .replace("{{0}}", "f32.convert_i32_u")
        .replace("{{1}}", "i32")
        .replace("{{2}}", "f32");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        1.0_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1)
            .unwrap()
    );
    assert_eq!(
        0.0_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0)
            .unwrap()
    );
    assert_eq!(
        2147483648_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 2147483647)
            .unwrap()
    );
    assert_eq!(
        2147483648_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -2147483648)
            .unwrap()
    );
    assert_eq!(
        hexf32!("0x1.234568p+28"),
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0x12345678)
            .unwrap()
    );
    assert_eq!(
        4294967296.0_f32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0xffffffff_u32 as i32
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("0x1.000000p+31"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0x80000080_u32 as i32
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("0x1.000002p+31"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0x80000081_u32 as i32
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("0x1.000002p+31"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0x80000082_u32 as i32
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("0x1.fffffcp+31"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0xfffffe80_u32 as i32
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("0x1.fffffep+31"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0xfffffe81_u32 as i32
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("0x1.fffffep+31"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0xfffffe82_u32 as i32
            )
            .unwrap()
    );
    assert_eq!(
        16777216.0_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 16777217)
            .unwrap()
    );
    assert_eq!(
        16777220.0_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 16777219)
            .unwrap()
    );
}

#[should_panic(expected = "validation failed: InvalidValidationStackValType(Some(NumType(I32)))")]
#[test_log::test]
pub fn f32_convert_i64_s_let_it_die() {
    let wat = String::from(WAT)
        .replace("{{0}}", "f32.convert_i64_s")
        .replace("{{1}}", "i32")
        .replace("{{2}}", "f32");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        -1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1)
            .unwrap()
    );
}

#[test_log::test]
pub fn f32_convert_i64_s() {
    let wat = String::from(WAT)
        .replace("{{0}}", "f32.convert_i64_s")
        .replace("{{1}}", "i64")
        .replace("{{2}}", "f32");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        1.0_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1_i64)
            .unwrap()
    );
    assert_eq!(
        -1.0_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1_i64)
            .unwrap()
    );
    assert_eq!(
        0.0_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0_i64)
            .unwrap()
    );
    assert_eq!(
        9223372036854775807_f32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                9223372036854775807_i64
            )
            .unwrap()
    );
    assert_eq!(
        -9223372036854775808_f32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                -9223372036854775808_i64
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("0x1.1db9e8p+48"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                314159265358979_i64
            )
            .unwrap()
    );
    assert_eq!(
        16777216.0_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 16777217_i64)
            .unwrap()
    );
    assert_eq!(
        -16777216.0_f32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                -16777217_i64
            )
            .unwrap()
    );
    assert_eq!(
        16777220.0_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 16777219_i64)
            .unwrap()
    );
    assert_eq!(
        -16777220.0_f32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                -16777219_i64
            )
            .unwrap()
    );

    assert_eq!(
        hexf32!("0x1.fffffep+62"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0x7fffff4000000001_i64
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("-0x1.fffffep+62"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0x8000004000000001_u64 as i64
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("0x1.000002p+53"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0x0020000020000001_i64
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("-0x1.000002p+53"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0xffdfffffdfffffff_u64 as i64
            )
            .unwrap()
    );
}

/// A function to test the f32.convert_i64_s implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/7570678ade1244ae69c9fefc990f4534c63ffaec/conversions.wast#L459)
#[test_log::test]
pub fn f32_convert_i64_u() {
    let wat = String::from(WAT)
        .replace("{{0}}", "f32.convert_i64_u")
        .replace("{{1}}", "i64")
        .replace("{{2}}", "f32");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        1.0_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1_i64)
            .unwrap()
    );
    assert_eq!(
        0.0_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0_i64)
            .unwrap()
    );
    assert_eq!(
        9223372036854775807_f32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                9223372036854775807_i64
            )
            .unwrap()
    );
    assert_eq!(
        9223372036854775808_f32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                -9223372036854775808_i64
            )
            .unwrap()
    );
    assert_eq!(
        18446744073709551616.0_f32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0xffffffffffffffff_u64 as i64
            )
            .unwrap()
    );
    // ;; Test rounding directions.
    assert_eq!(
        16777216.0_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 16777217_i64)
            .unwrap()
    );
    assert_eq!(
        16777220.0_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 16777219_i64)
            .unwrap()
    );

    assert_eq!(
        hexf32!("0x1.000002p+53"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0x0020000020000001_i64
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("0x1.fffffep+62"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0x7fffffbfffffffff_i64
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("0x1.000002p+63"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0x8000008000000001_u64 as i64
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("0x1.fffffep+63"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0xfffffe8000000001_u64 as i64
            )
            .unwrap()
    );
}

#[should_panic(expected = "validation failed: InvalidValidationStackValType(Some(NumType(F32)))")]
#[test_log::test]
pub fn f32_demote_f64_let_it_die() {
    let wat = String::from(WAT)
        .replace("{{0}}", "f32.demote_f64")
        .replace("{{1}}", "f32")
        .replace("{{2}}", "f32");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        0.0_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0.0_f64)
            .unwrap()
    );
}

/// A function to test the f32.demote_f64 implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/7570678ade1244ae69c9fefc990f4534c63ffaec/conversions.wast#L565)
#[test_log::test]
pub fn f32_demote_f64() {
    let wat = String::from(WAT)
        .replace("{{0}}", "f32.demote_f64")
        .replace("{{1}}", "f64")
        .replace("{{2}}", "f32");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        0.0_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0.0_f64)
            .unwrap()
    );
    assert_eq!(
        -0.0_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -0.0_f64)
            .unwrap()
    );
    assert_eq!(
        0.0_f32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("0x0.0000000000001p-1022")
            )
            .unwrap()
    );
    assert_eq!(
        -0.0_f32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("-0x0.0000000000001p-1022")
            )
            .unwrap()
    );
    assert_eq!(
        1.0_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1.0_f64)
            .unwrap()
    );
    assert_eq!(
        -1.0_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1.0_f64)
            .unwrap()
    );
    assert_eq!(
        hexf32!("0x1p-126"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("0x1.fffffe0000000p-127")
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("-0x1p-126"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("-0x1.fffffe0000000p-127")
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("0x1.fffffcp-127"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("0x1.fffffdfffffffp-127")
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("-0x1.fffffcp-127"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("-0x1.fffffdfffffffp-127")
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("0x1p-149"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("0x1p-149")
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("-0x1p-149"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("-0x1p-149")
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("0x1.fffffcp+127"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("0x1.fffffd0000000p+127")
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("-0x1.fffffcp+127"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("-0x1.fffffd0000000p+127")
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("0x1.fffffep+127"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("0x1.fffffd0000001p+127")
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("-0x1.fffffep+127"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("-0x1.fffffd0000001p+127")
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("0x1.fffffep+127"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("0x1.fffffep+127")
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("-0x1.fffffep+127"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("-0x1.fffffep+127")
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("0x1.fffffep+127"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("0x1.fffffefffffffp+127")
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("-0x1.fffffep+127"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("-0x1.fffffefffffffp+127")
            )
            .unwrap()
    );
    assert_eq!(
        f32::INFINITY,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("0x1.ffffffp+127")
            )
            .unwrap()
    );
    assert_eq!(
        f32::NEG_INFINITY,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("-0x1.ffffffp+127")
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("0x1p-119"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("0x1p-119")
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("0x1.8f867ep+125"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("0x1.8f867ep+125")
            )
            .unwrap()
    );
    assert_eq!(
        f32::INFINITY,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                f64::INFINITY
            )
            .unwrap()
    );
    assert_eq!(
        f32::NEG_INFINITY,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                f64::NEG_INFINITY
            )
            .unwrap()
    );
    assert_eq!(
        1.0_f32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("0x1.0000000000001p+0")
            )
            .unwrap()
    );
    assert_eq!(
        1.0_f32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("0x1.fffffffffffffp-1")
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("0x1.000000p+0"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("0x1.0000010000000p+0")
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("0x1.000002p+0"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("0x1.0000010000001p+0")
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("0x1.000002p+0"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("0x1.000002fffffffp+0")
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("0x1.000004p+0"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("0x1.0000030000000p+0")
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("0x1.000004p+0"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("0x1.0000050000000p+0")
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("0x1.0p+24"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("0x1.0000010000000p+24")
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("0x1.000002p+24"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("0x1.0000010000001p+24")
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("0x1.000002p+24"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("0x1.000002fffffffp+24")
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("0x1.000004p+24"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("0x1.0000030000000p+24")
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("0x1.4eae5p+108"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("0x1.4eae4f7024c7p+108")
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("0x1.a12e72p-113"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("0x1.a12e71e358685p-113")
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("0x1.cb9834p-127"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("0x1.cb98354d521ffp-127")
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("-0x1.6972b4p+1"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("-0x1.6972b30cfb562p+1")
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("-0x1.bedbe4p+112"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("-0x1.bedbe4819d4c4p+112")
            )
            .unwrap()
    );
    // assert_eq!(f32::NAN, instance.invoke(&instance.get_fn_idx(0, 0).unwrap(), f64::NAN).unwrap());
    // (assert_return (invoke "f32.demote_f64" (f64.const nan)) (f32.const nan:canonical))
    // (assert_return (invoke "f32.demote_f64" (f64.const nan:0x4000000000000)) (f32.const nan:arithmetic))
    // assert_eq!(f32::NAN, instance.invoke(&instance.get_fn_idx(0, 0).unwrap(), -f64::NAN).unwrap());
    // (assert_return (invoke "f32.demote_f64" (f64.const -nan)) (f32.const nan:canonical))
    // (assert_return (invoke "f32.demote_f64" (f64.const -nan:0x4000000000000)) (f32.const nan:arithmetic))
    assert_eq!(
        0.0_f32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("0x1p-1022")
            )
            .unwrap()
    );
    assert_eq!(
        -0.0_f32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("-0x1p-1022")
            )
            .unwrap()
    );
    assert_eq!(
        0.0_f32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("0x1.0p-150")
            )
            .unwrap()
    );
    assert_eq!(
        -0.0_f32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("-0x1.0p-150")
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("0x1p-149"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("0x1.0000000000001p-150")
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("-0x1p-149"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("-0x1.0000000000001p-150")
            )
            .unwrap()
    );
}

#[should_panic(expected = "validation failed: InvalidValidationStackValType(Some(NumType(I64)))")]
#[test_log::test]
pub fn f64_convert_i32_s_let_it_die() {
    let wat = String::from(WAT)
        .replace("{{0}}", "f64.convert_i32_s")
        .replace("{{1}}", "i64")
        .replace("{{2}}", "f64");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        -1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1)
            .unwrap()
    );
}

/// A function to test the f64.convert_i32_s implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/7570678ade1244ae69c9fefc990f4534c63ffaec/conversions.wast#L476)
#[test_log::test]
pub fn f64_convert_i32_s() {
    let wat = String::from(WAT)
        .replace("{{0}}", "f64.convert_i32_s")
        .replace("{{1}}", "i32")
        .replace("{{2}}", "f64");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        1.0_f64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1)
            .unwrap()
    );
    assert_eq!(
        -1.0_f64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1)
            .unwrap()
    );
    assert_eq!(
        0.0_f64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0)
            .unwrap()
    );
    assert_eq!(
        2147483647_f64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 2147483647)
            .unwrap()
    );
    assert_eq!(
        -2147483648_f64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -2147483648)
            .unwrap()
    );
    assert_eq!(
        987654321_f64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 987654321)
            .unwrap()
    );
}

/// A function to test the f64.convert_i32_u implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/7570678ade1244ae69c9fefc990f4534c63ffaec/conversions.wast#L525)
#[test_log::test]
pub fn f64_convert_i32_u() {
    let wat = String::from(WAT)
        .replace("{{0}}", "f64.convert_i32_u")
        .replace("{{1}}", "i32")
        .replace("{{2}}", "f64");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        1.0_f64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1)
            .unwrap()
    );
    assert_eq!(
        0.0_f64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0)
            .unwrap()
    );
    assert_eq!(
        2147483647_f64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 2147483647)
            .unwrap()
    );
    assert_eq!(
        2147483648_f64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -2147483648)
            .unwrap()
    );
    assert_eq!(
        4294967295.0_f64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0xffffffff_u32 as i32
            )
            .unwrap()
    );
}

#[should_panic(expected = "validation failed: InvalidValidationStackValType(Some(NumType(I32)))")]
#[test_log::test]
pub fn f64_convert_i64_s_let_it_die() {
    let wat = String::from(WAT)
        .replace("{{0}}", "f64.convert_i64_s")
        .replace("{{1}}", "i32")
        .replace("{{2}}", "f64");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        -1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1)
            .unwrap()
    );
}

/// A function to test the f64.convert_i64_s implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/7570678ade1244ae69c9fefc990f4534c63ffaec/conversions.wast#L483)
#[test_log::test]
pub fn f64_convert_i64_s() {
    let wat = String::from(WAT)
        .replace("{{0}}", "f64.convert_i64_s")
        .replace("{{1}}", "i64")
        .replace("{{2}}", "f64");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        1.0_f64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1_i64)
            .unwrap()
    );
    assert_eq!(
        -1.0_f64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1_i64)
            .unwrap()
    );
    assert_eq!(
        0.0_f64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0_i64)
            .unwrap()
    );
    assert_eq!(
        9223372036854775807_f64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                9223372036854775807_i64
            )
            .unwrap()
    );
    assert_eq!(
        -9223372036854775808_f64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                -9223372036854775808_i64
            )
            .unwrap()
    );
    assert_eq!(
        4669201609102990_f64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                4669201609102990_i64
            )
            .unwrap()
    );
    assert_eq!(
        9007199254740992_f64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                9007199254740993_i64
            )
            .unwrap()
    );
    assert_eq!(
        -9007199254740992_f64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                -9007199254740993_i64
            )
            .unwrap()
    );
    assert_eq!(
        9007199254740996_f64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                9007199254740995_i64
            )
            .unwrap()
    );
    assert_eq!(
        -9007199254740996_f64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                -9007199254740995_i64
            )
            .unwrap()
    );
}

/// A function to test the f64.convert_i64_u implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/7570678ade1244ae69c9fefc990f4534c63ffaec/conversions.wast#L531C1-L544C103)
#[test_log::test]
pub fn f64_convert_i64_u() {
    let wat = String::from(WAT)
        .replace("{{0}}", "f64.convert_i64_u")
        .replace("{{1}}", "i64")
        .replace("{{2}}", "f64");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        1.0_f64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1_i64)
            .unwrap()
    );
    assert_eq!(
        0.0_f64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0_i64)
            .unwrap()
    );
    assert_eq!(
        9223372036854775807_f64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                9223372036854775807_i64
            )
            .unwrap()
    );
    assert_eq!(
        9223372036854775808_f64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                -9223372036854775808_i64
            )
            .unwrap()
    );
    assert_eq!(
        18446744073709551616.0_f64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0xffffffffffffffff_u64 as i64
            )
            .unwrap()
    );
    assert_eq!(
        hexf64!("0x1.0000000000000p+63"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0x8000000000000400_u64 as i64
            )
            .unwrap()
    );
    assert_eq!(
        hexf64!("0x1.0000000000001p+63"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0x8000000000000401_u64 as i64
            )
            .unwrap()
    );
    assert_eq!(
        hexf64!("0x1.0000000000001p+63"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0x8000000000000402_u64 as i64
            )
            .unwrap()
    );
    assert_eq!(
        hexf64!("0x1.ffffffffffffep+63"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0xfffffffffffff400_u64 as i64
            )
            .unwrap()
    );
    assert_eq!(
        hexf64!("0x1.fffffffffffffp+63"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0xfffffffffffff401_u64 as i64
            )
            .unwrap()
    );
    assert_eq!(
        hexf64!("0x1.fffffffffffffp+63"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0xfffffffffffff402_u64 as i64
            )
            .unwrap()
    );
    // ;; Test rounding directions.
    assert_eq!(
        9007199254740992_f64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                9007199254740993_i64
            )
            .unwrap()
    );
    assert_eq!(
        9007199254740996_f64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                9007199254740995_i64
            )
            .unwrap()
    );
}

#[should_panic(expected = "validation failed: InvalidValidationStackValType(Some(NumType(I64)))")]
#[test_log::test]
pub fn f64_promote_f32_let_it_die() {
    let wat = String::from(WAT)
        .replace("{{0}}", "f64.promote_f32")
        .replace("{{1}}", "i64")
        .replace("{{2}}", "f64");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        -1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1)
            .unwrap()
    );
}

/// A function to test the f64.promote_f32 implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/7570678ade1244ae69c9fefc990f4534c63ffaec/conversions.wast#L546)
#[test_log::test]
pub fn f64_promote_f32() {
    let wat = String::from(WAT)
        .replace("{{0}}", "f64.promote_f32")
        .replace("{{1}}", "f32")
        .replace("{{2}}", "f64");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        0.0_f64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0.0_f32)
            .unwrap()
    );
    assert_eq!(
        -0.0_f64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -0.0_f32)
            .unwrap()
    );
    assert_eq!(
        hexf64!("0x1p-149"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf32!("0x1p-149")
            )
            .unwrap()
    );
    assert_eq!(
        hexf64!("-0x1p-149"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf32!("-0x1p-149")
            )
            .unwrap()
    );
    assert_eq!(
        1.0_f64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1.0_f32)
            .unwrap()
    );
    assert_eq!(
        -1.0_f64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1.0_f32)
            .unwrap()
    );
    assert_eq!(
        hexf64!("-0x1.fffffep+127"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf32!("-0x1.fffffep+127")
            )
            .unwrap()
    );
    assert_eq!(
        hexf64!("0x1.fffffep+127"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf32!("0x1.fffffep+127")
            )
            .unwrap()
    );
    assert_eq!(
        hexf64!("0x1p-119"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf32!("0x1p-119")
            )
            .unwrap()
    );
    assert_eq!(
        6.638_253_671_010_439_5e37_f64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf32!("0x1.8f867ep+125")
            )
            .unwrap()
    );
    assert_eq!(
        f64::INFINITY,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                f32::INFINITY
            )
            .unwrap()
    );
    assert_eq!(
        f64::NEG_INFINITY,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                f32::NEG_INFINITY
            )
            .unwrap()
    );
    // (assert_return (invoke "f64.promote_f32" (f32.const nan)) (f64.const nan:canonical))
    // (assert_return (invoke "f64.promote_f32" (f32.const nan:0x200000)) (f64.const nan:arithmetic))
    // (assert_return (invoke "f64.promote_f32" (f32.const -nan)) (f64.const nan:canonical))
    // (assert_return (invoke "f64.promote_f32" (f32.const -nan:0x200000)) (f64.const nan:arithmetic))
}

#[should_panic(expected = "validation failed: InvalidValidationStackValType(Some(NumType(I64)))")]
#[test_log::test]
pub fn i32_reinterpret_f32_let_it_die() {
    let wat = String::from(WAT)
        .replace("{{0}}", "i32.reinterpret_f32")
        .replace("{{1}}", "i64")
        .replace("{{2}}", "f32");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        -1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1)
            .unwrap()
    );
}

/// A function to test the i32.reinterpret_f32 implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/7570678ade1244ae69c9fefc990f4534c63ffaec/conversions.wast#L644)
#[test_log::test]
pub fn i32_reinterpret_f32() {
    let wat = String::from(WAT)
        .replace("{{0}}", "i32.reinterpret_f32")
        .replace("{{1}}", "f32")
        .replace("{{2}}", "i32");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0.0_f32)
            .unwrap()
    );
    assert_eq!(
        0x80000000_u32 as i32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -0.0_f32)
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf32!("0x1p-149")
            )
            .unwrap()
    );
    // (assert_return (invoke "i32.reinterpret_f32" (f32.const -nan:0x7fffff)) (i32.const -1))
    assert_eq!(
        0x80000001_u32 as i32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf32!("-0x1p-149")
            )
            .unwrap()
    );
    assert_eq!(
        1065353216,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1.0_f32)
            .unwrap()
    );
    assert_eq!(
        1078530010,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                3.1415926_f32
            )
            .unwrap()
    );
    assert_eq!(
        2139095039,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf32!("0x1.fffffep+127")
            )
            .unwrap()
    );
    assert_eq!(
        -8388609,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf32!("-0x1.fffffep+127")
            )
            .unwrap()
    );
    assert_eq!(
        0x7f800000,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                f32::INFINITY
            )
            .unwrap()
    );
    assert_eq!(
        0xff800000_u32 as i32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                f32::NEG_INFINITY
            )
            .unwrap()
    );
    assert_eq!(
        0x7fc00000,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), f32::NAN)
            .unwrap()
    );
    assert_eq!(
        0xffc00000_u32 as i32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -f32::NAN)
            .unwrap()
    );
    // (assert_return (invoke "i32.reinterpret_f32" (f32.const -nan)) (i32.const 0xffc00000))
    // (assert_return (invoke "i32.reinterpret_f32" (f32.const nan:0x200000)) (i32.const 0x7fa00000))
    // (assert_return (invoke "i32.reinterpret_f32" (f32.const -nan:0x200000)) (i32.const 0xffa00000))
}

#[should_panic(expected = "validation failed: InvalidValidationStackValType(Some(NumType(I32)))")]
#[test_log::test]
pub fn i64_reinterpret_f64_let_it_die() {
    let wat = String::from(WAT)
        .replace("{{0}}", "i64.reinterpret_f64")
        .replace("{{1}}", "i32")
        .replace("{{2}}", "f64");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        -1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1)
            .unwrap()
    );
}

/// A function to test the i64.reinterpret_f64 implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/7570678ade1244ae69c9fefc990f4534c63ffaec/conversions.wast#L660)
#[test_log::test]
pub fn i64_reinterpret_f64() {
    let wat = String::from(WAT)
        .replace("{{0}}", "i64.reinterpret_f64")
        .replace("{{1}}", "f64")
        .replace("{{2}}", "i64");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        0_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0.0_f64)
            .unwrap()
    );
    assert_eq!(
        0x8000000000000000_u64 as i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -0.0_f64)
            .unwrap()
    );
    assert_eq!(
        1_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("0x0.0000000000001p-1022")
            )
            .unwrap()
    );
    // (assert_return (invoke "i64.reinterpret_f64" (f64.const -nan:0xfffffffffffff)) (i64.const -1))
    assert_eq!(
        0x8000000000000001_u64 as i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("-0x0.0000000000001p-1022")
            )
            .unwrap()
    );
    assert_eq!(
        4607182418800017408_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1.0_f64)
            .unwrap()
    );
    assert_eq!(
        4614256656552045841_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                3.14159265358979_f64
            )
            .unwrap()
    );
    assert_eq!(
        9218868437227405311_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("0x1.fffffffffffffp+1023")
            )
            .unwrap()
    );
    assert_eq!(
        -4503599627370497_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf64!("-0x1.fffffffffffffp+1023")
            )
            .unwrap()
    );
    assert_eq!(
        0x7ff0000000000000_u64 as i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                f64::INFINITY
            )
            .unwrap()
    );
    assert_eq!(
        0xfff0000000000000_u64 as i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                f64::NEG_INFINITY
            )
            .unwrap()
    );
    assert_eq!(
        0x7ff8000000000000_u64 as i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), f64::NAN)
            .unwrap()
    );
    assert_eq!(
        0xfff8000000000000_u64 as i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -f64::NAN)
            .unwrap()
    );
    // (assert_return (invoke "i64.reinterpret_f64" (f64.const nan:0x4000000000000)) (i64.const 0x7ff4000000000000))
    // (assert_return (invoke "i64.reinterpret_f64" (f64.const -nan:0x4000000000000)) (i64.const 0xfff4000000000000))
}

#[should_panic(expected = "validation failed: InvalidValidationStackValType(Some(NumType(I64)))")]
#[test_log::test]
pub fn f32_reinterpret_i32_let_it_die() {
    let wat = String::from(WAT)
        .replace("{{0}}", "f32.reinterpret_i32")
        .replace("{{1}}", "i64")
        .replace("{{2}}", "f32");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        -1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1)
            .unwrap()
    );
}

/// A function to test the f32.reinterpret_i32 implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/7570678ade1244ae69c9fefc990f4534c63ffaec/conversions.wast#L618)
#[test_log::test]
pub fn f32_reinterpret_i32() {
    let wat = String::from(WAT)
        .replace("{{0}}", "f32.reinterpret_i32")
        .replace("{{1}}", "i32")
        .replace("{{2}}", "f32");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        0.0_f32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0)
            .unwrap()
    );
    assert_eq!(
        -0.0_f32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0x80000000_u32 as i32
            )
            .unwrap()
    );
    assert_eq!(
        hexf32!("0x1p-149"),
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1)
            .unwrap()
    );
    // (assert_return (invoke "f32.reinterpret_i32" (i32.const -1)) (f32.const -nan:0x7fffff))
    assert_eq!(
        hexf32!("0x1.b79a2ap-113"),
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 123456789)
            .unwrap()
    );
    assert_eq!(
        hexf32!("-0x1p-149"),
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -2147483647)
            .unwrap()
    );
    assert_eq!(
        f32::INFINITY,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0x7f800000)
            .unwrap()
    );
    assert_eq!(
        f32::NEG_INFINITY,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0xff800000_u32 as i32
            )
            .unwrap()
    );
    {
        let result: f32 = instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0x7fc00000)
            .unwrap();
        assert!(result.is_nan());
        assert!(result.is_sign_positive());
    }
    {
        let result: f32 = instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0xffc00000_u32 as i32,
            )
            .unwrap();
        assert!(result.is_nan());
        assert!(result.is_sign_negative());
    }
    // (assert_return (invoke "f32.reinterpret_i32" (i32.const 0x7fa00000)) (f32.const nan:0x200000))
    // (assert_return (invoke "f32.reinterpret_i32" (i32.const 0xffa00000)) (f32.const -nan:0x200000))
}

#[should_panic(expected = "validation failed: InvalidValidationStackValType(Some(NumType(I32)))")]
#[test_log::test]
pub fn f64_reinterpret_i64_let_it_die() {
    let wat = String::from(WAT)
        .replace("{{0}}", "f64.reinterpret_i64")
        .replace("{{1}}", "i32")
        .replace("{{2}}", "f64");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        -1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1)
            .unwrap()
    );
}

/// A function to test the i64.reinterpret_f64 implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/7570678ade1244ae69c9fefc990f4534c63ffaec/conversions.wast#L660)
#[test_log::test]
pub fn f64_reinterpret_i64() {
    let wat = String::from(WAT)
        .replace("{{0}}", "f64.reinterpret_i64")
        .replace("{{1}}", "i64")
        .replace("{{2}}", "f64");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        0.0_f64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0_i64)
            .unwrap()
    );
    assert_eq!(
        hexf64!("0x0.0000000000001p-1022"),
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1_i64)
            .unwrap()
    );
    // (assert_return (invoke "f64.reinterpret_i64" (i64.const -1)) (f64.const -nan:0xfffffffffffff))
    assert_eq!(
        -0.0_f64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0x8000000000000000_u64 as i64
            )
            .unwrap()
    );
    assert_eq!(
        hexf64!("0x0.00000499602d2p-1022"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                1234567890_i64
            )
            .unwrap()
    );
    assert_eq!(
        hexf64!("-0x0.0000000000001p-1022"),
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                -9223372036854775807_i64
            )
            .unwrap()
    );
    assert_eq!(
        f64::INFINITY,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0x7ff0000000000000_u64 as i64
            )
            .unwrap()
    );
    assert_eq!(
        f64::NEG_INFINITY,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0xfff0000000000000_u64 as i64
            )
            .unwrap()
    );
    {
        let result: f64 = instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0x7ff8000000000000_u64 as i64,
            )
            .unwrap();
        assert!(result.is_nan());
        assert!(result.is_sign_positive());
    }
    {
        let result: f64 = instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0xfff8000000000000_u64 as i64,
            )
            .unwrap();
        assert!(result.is_nan());
        assert!(result.is_sign_negative());
    }
    // (assert_return (invoke "f64.reinterpret_i64" (i64.const 0x7ff4000000000000)) (f64.const nan:0x4000000000000))
    // (assert_return (invoke "f64.reinterpret_i64" (i64.const 0xfff4000000000000)) (f64.const -nan:0x4000000000000))
}
