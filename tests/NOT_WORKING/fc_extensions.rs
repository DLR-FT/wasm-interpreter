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
use wasm::{validate, RuntimeInstance};

const WAT: &str = r#"
      (module
      (func (export "{{0}}") (param $x {{1}}) (result {{2}})
          local.get $x
          {{0}})
    )
"#;

#[should_panic(expected = "validation failed: InvalidValidationStackValType(Some(NumType(F64)))")]
#[test_log::test]
pub fn i32_trunc_sat_f32_s_let_it_die() {
    let wat = String::from(WAT)
        .replace("{{0}}", "i32.trunc_sat_f32_s")
        .replace("{{1}}", "f64")
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

#[test_log::test]
pub fn i32_trunc_sat_f32_s() {
    let wat = String::from(WAT)
        .replace("{{0}}", "i32.trunc_sat_f32_s")
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
        -2147483648,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                -2147483648.0_f32
            )
            .unwrap()
    );
    assert_eq!(
        0x7fffffff,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                2147483648.0_f32
            )
            .unwrap()
    );
    assert_eq!(
        0x80000000_u32 as i32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                -2147483904.0_f32
            )
            .unwrap()
    );
    assert_eq!(
        0x7fffffff,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                f32::INFINITY
            )
            .unwrap()
    );
    assert_eq!(
        0x80000000_u32 as i32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                -f32::INFINITY
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), f32::NAN)
            .unwrap()
    );
    // (assert_return (invoke "i32.trunc_sat_f32_s" (f32.const nan:0x200000)) (i32.const 0))
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -f32::NAN)
            .unwrap()
    );
    // (assert_return (invoke "i32.trunc_sat_f32_s" (f32.const -nan:0x200000)) (i32.const 0))
}

#[should_panic(expected = "validation failed: InvalidValidationStackValType(Some(NumType(F64)))")]
#[test_log::test]
pub fn i32_trunc_sat_f32_u_let_it_die() {
    let wat = String::from(WAT)
        .replace("{{0}}", "i32.trunc_sat_f32_u")
        .replace("{{1}}", "f64")
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

#[test_log::test]
pub fn i32_trunc_sat_f32_u() {
    let wat = String::from(WAT)
        .replace("{{0}}", "i32.trunc_sat_f32_u")
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
        0xffffffff_u32 as i32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                4294967296.0_f32
            )
            .unwrap()
    );
    assert_eq!(
        0x00000000,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1_f32)
            .unwrap()
    );
    assert_eq!(
        0xffffffff_u32 as i32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                f32::INFINITY
            )
            .unwrap()
    );
    assert_eq!(
        0x00000000,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                f32::NEG_INFINITY
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), f32::NAN)
            .unwrap()
    );
    // (assert_return (invoke "i32.trunc_sat_f32_u" (f32.const nan:0x200000)) (i32.const 0))
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -f32::NAN)
            .unwrap()
    );
    // (assert_return (invoke "i32.trunc_sat_f32_u" (f32.const -nan:0x200000)) (i32.const 0))
}

#[should_panic(expected = "validation failed: InvalidValidationStackValType(Some(NumType(F32)))")]
#[test_log::test]
pub fn i32_trunc_sat_f64_s_let_it_die() {
    let wat = String::from(WAT)
        .replace("{{0}}", "i32.trunc_sat_f64_s")
        .replace("{{1}}", "f32")
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

#[test_log::test]
pub fn i32_trunc_sat_f64_s() {
    let wat = String::from(WAT)
        .replace("{{0}}", "i32.trunc_sat_f64_s")
        .replace("{{1}}", "f64")
        .replace("{{2}}", "i32");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0.0)
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -0.0)
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
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1.0)
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
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1.5)
            .unwrap()
    );
    assert_eq!(
        -1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1.0)
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
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1.5)
            .unwrap()
    );
    assert_eq!(
        -1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1.9)
            .unwrap()
    );
    assert_eq!(
        -2,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -2.0)
            .unwrap()
    );
    assert_eq!(
        2147483647,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 2147483647.0)
            .unwrap()
    );
    assert_eq!(
        -2147483648,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                -2147483648.0
            )
            .unwrap()
    );
    assert_eq!(
        0x7fffffff,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 2147483648.0)
            .unwrap()
    );
    assert_eq!(
        0x80000000_u32 as i32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                -2147483649.0
            )
            .unwrap()
    );
    assert_eq!(
        0x7fffffff,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                f64::INFINITY
            )
            .unwrap()
    );
    assert_eq!(
        0x80000000_u32 as i32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                -f64::INFINITY
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), f64::NAN)
            .unwrap()
    );
    // (assert_return (invoke "i32.trunc_sat_s" (f32.const nan:0x200000)) (i32.const 0))
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -f64::NAN)
            .unwrap()
    );
    // (assert_return (invoke "i32.trunc_sat_s" (f32.const -nan:0x200000)) (i32.const 0))
}

#[should_panic(expected = "validation failed: InvalidValidationStackValType(Some(NumType(F32)))")]
#[test_log::test]
pub fn i32_trunc_sat_f64_u_let_it_die() {
    let wat = String::from(WAT)
        .replace("{{0}}", "i32.trunc_sat_f64_u")
        .replace("{{1}}", "f32")
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

#[test_log::test]
pub fn i32_trunc_sat_f64_u() {
    let wat = String::from(WAT)
        .replace("{{0}}", "i32.trunc_sat_f64_u")
        .replace("{{1}}", "f64")
        .replace("{{2}}", "i32");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0.0)
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -0.0)
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
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1.0)
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
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1.5)
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1.9)
            .unwrap()
    );
    assert_eq!(
        2,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 2.0)
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
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 4294967295.0)
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
        0xffffffff_u32 as i32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 4294967296.0)
            .unwrap()
    );
    assert_eq!(
        0x00000000,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1.0)
            .unwrap()
    );
    assert_eq!(
        0xffffffff_u32 as i32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1e16_f64)
            .unwrap()
    );
    assert_eq!(
        0xffffffff_u32 as i32,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1e30_f64)
            .unwrap()
    );
    assert_eq!(
        0xffffffff_u32 as i32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                9223372036854775808_f64
            )
            .unwrap()
    );
    assert_eq!(
        0xffffffff_u32 as i32,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                f64::INFINITY
            )
            .unwrap()
    );
    assert_eq!(
        0x00000000,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                f64::NEG_INFINITY
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), f64::NAN)
            .unwrap()
    );
    // (assert_return (invoke "i32.trunc_sat_f64_u" (f64.const nan:0x4000000000000)) (i32.const 0))
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -f64::NAN)
            .unwrap()
    );
    // (assert_return (invoke "i32.trunc_sat_f64_u" (f64.const -nan:0x4000000000000)) (i32.const 0))
}

#[should_panic(expected = "validation failed: InvalidValidationStackValType(Some(NumType(F64)))")]
#[test_log::test]
pub fn i64_trunc_sat_f32_s_let_it_die() {
    let wat = String::from(WAT)
        .replace("{{0}}", "i64.trunc_sat_f32_s")
        .replace("{{1}}", "f64")
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

#[test_log::test]
pub fn i64_trunc_sat_f32_s() {
    let wat = String::from(WAT)
        .replace("{{0}}", "i64.trunc_sat_f32_s")
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
        0x7fffffffffffffff_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                9223372036854775808.0_f32
            )
            .unwrap()
    );
    assert_eq!(
        0x8000000000000000_u64 as i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                -9223373136366403584.0_f32
            )
            .unwrap()
    );
    assert_eq!(
        0x7fffffffffffffff_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                f32::INFINITY
            )
            .unwrap()
    );
    assert_eq!(
        0x8000000000000000_u64 as i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                -f32::INFINITY
            )
            .unwrap()
    );
    assert_eq!(
        0_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), f32::NAN)
            .unwrap()
    );
    // (assert_return (invoke "i64.trunc_sat_f32_s" (f32.const nan:0x200000)) (i64.const 0))
    assert_eq!(
        0_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -f32::NAN)
            .unwrap()
    );
    // (assert_return (invoke "i64.trunc_sat_f32_s" (f32.const -nan:0x200000)) (i64.const 0))
}

#[should_panic(expected = "validation failed: InvalidValidationStackValType(Some(NumType(F64)))")]
#[test_log::test]
pub fn i64_trunc_sat_f32_u_let_it_die() {
    let wat = String::from(WAT)
        .replace("{{0}}", "i64.trunc_sat_f32_u")
        .replace("{{1}}", "f64")
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

#[test_log::test]
pub fn i64_trunc_sat_f32_u() {
    let wat = String::from(WAT)
        .replace("{{0}}", "i64.trunc_sat_f32_u")
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
        0_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                hexf32!("-0x1.ccccccp-1")
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
        0xffffffffffffffff_u64 as i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                18446744073709551616.0_f32
            )
            .unwrap()
    );
    assert_eq!(
        0x0000000000000000_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1.0_f32)
            .unwrap()
    );
    assert_eq!(
        0xffffffffffffffff_u64 as i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                f32::INFINITY
            )
            .unwrap()
    );
    assert_eq!(
        0x0000000000000000_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                f32::NEG_INFINITY
            )
            .unwrap()
    );
    assert_eq!(
        0_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), f32::NAN)
            .unwrap()
    );
    // (assert_return (invoke "i64.trunc_sat_f32_u" (f32.const nan:0x200000)) (i64.const 0))
    assert_eq!(
        0_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -f32::NAN)
            .unwrap()
    );
    // (assert_return (invoke "i64.trunc_sat_f32_u" (f32.const -nan:0x200000)) (i64.const 0))
}

#[should_panic(expected = "validation failed: InvalidValidationStackValType(Some(NumType(F32)))")]
#[test_log::test]
pub fn i64_trunc_sat_f64_s_let_it_die() {
    let wat = String::from(WAT)
        .replace("{{0}}", "i64.trunc_sat_f64_s")
        .replace("{{1}}", "f32")
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

#[test_log::test]
pub fn i64_trunc_sat_f64_s() {
    let wat = String::from(WAT)
        .replace("{{0}}", "i64.trunc_sat_f64_s")
        .replace("{{1}}", "f64")
        .replace("{{2}}", "i64");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        0_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0.0)
            .unwrap()
    );
    assert_eq!(
        0_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -0.0)
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
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1.0)
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
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1.5)
            .unwrap()
    );
    assert_eq!(
        -1_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1.0)
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
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1.5)
            .unwrap()
    );
    assert_eq!(
        -1_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1.9)
            .unwrap()
    );
    assert_eq!(
        -2_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -2.0)
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
                9223372036854774784.0
            )
            .unwrap()
    );
    assert_eq!(
        -9223372036854775808_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                -9223372036854775808.0
            )
            .unwrap()
    );
    assert_eq!(
        0x7fffffffffffffff_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                9223372036854775808.0
            )
            .unwrap()
    );
    assert_eq!(
        0x8000000000000000_u64 as i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                -9223372036854777856.0
            )
            .unwrap()
    );
    assert_eq!(
        0x7fffffffffffffff_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                f64::INFINITY
            )
            .unwrap()
    );
    assert_eq!(
        0x8000000000000000_u64 as i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                -f64::INFINITY
            )
            .unwrap()
    );
    assert_eq!(
        0_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), f64::NAN)
            .unwrap()
    );
    // (assert_return (invoke "i64.trunc_sat_f64_s" (f64.const nan:0x4000000000000)) (i64.const 0))
    assert_eq!(
        0_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -f64::NAN)
            .unwrap()
    );
    // (assert_return (invoke "i64.trunc_sat_f64_s" (f64.const -nan:0x4000000000000)) (i64.const 0))
}

#[should_panic(expected = "validation failed: InvalidValidationStackValType(Some(NumType(F32)))")]
#[test_log::test]
pub fn i64_trunc_sat_f64_u_let_it_die() {
    let wat = String::from(WAT)
        .replace("{{0}}", "i64.trunc_sat_f64_u")
        .replace("{{1}}", "f32")
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

#[test_log::test]
pub fn i64_trunc_sat_f64_u() {
    let wat = String::from(WAT)
        .replace("{{0}}", "i64.trunc_sat_f64_u")
        .replace("{{1}}", "f64")
        .replace("{{2}}", "i64");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        0_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0.0)
            .unwrap()
    );
    assert_eq!(
        0_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -0.0)
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
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1.0)
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
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1.5)
            .unwrap()
    );
    assert_eq!(
        1_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1.9)
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
                18446744073709549568.0
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
        0xffffffffffffffff_u64 as i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                18446744073709551616.0_f64
            )
            .unwrap()
    );
    assert_eq!(
        0x0000000000000000_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -1.0)
            .unwrap()
    );
    assert_eq!(
        0xffffffffffffffff_u64 as i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                f64::INFINITY
            )
            .unwrap()
    );
    assert_eq!(
        0x0000000000000000_i64,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                f64::NEG_INFINITY
            )
            .unwrap()
    );
    assert_eq!(
        0_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), f64::NAN)
            .unwrap()
    );
    // (assert_return (invoke "i64.trunc_sat_f64_u" (f64.const nan:0x4000000000000)) (i64.const 0))
    assert_eq!(
        0_i64,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), -f64::NAN)
            .unwrap()
    );
    // (assert_return (invoke "i64.trunc_sat_f64_u" (f64.const -nan:0x4000000000000)) (i64.const 0))
}
