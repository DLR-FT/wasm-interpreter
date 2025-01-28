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
use wasm::{validate, RuntimeInstance};

const WAT: &str = r#"
      (module
      (func (export "i64_{{0}}") (param $x i64) (param $y i64) (result i32)
          local.get $x
          local.get $y
          i64.{{0}})
    )
"#;

#[should_panic]
#[test_log::test]
pub fn i64_eqz_panic() {
    let wat = r#"
  (module
    (func (export "i64_eqz") (result i32)
      i32.const 1
      i64.eqz
    )
  )
"#;

    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");

    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), ())
            .unwrap()
    );
}

/// A function to test the i64.eqz implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/5741d6c5172866174fde27c6b5447af757528d1a/i64.wast#L298)
#[test_log::test]
pub fn i64_eqz() {
    let wat = r#"
      (module
        (func (export "i64_eqz") (param $x i64) (result i32)
          local.get $x
          i64.eqz
        )
      )
    "#;

    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");

    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0_i64)
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1_i64)
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0x8000000000000000u64 as i64
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0x7fffffffffffffffu64 as i64
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0xffffffffffffffffu64 as i64
            )
            .unwrap()
    );
}

#[should_panic]
#[test_log::test]
pub fn i64_eq_panic_first_arg() {
    let wat = r#"
  (module
    (func (export "i64_eq") (result i32)
      i32.const 1
      i64.const 1
      i64.eq
    )
  )
"#;

    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");

    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), ())
            .unwrap()
    );
}

#[should_panic]
#[test_log::test]
pub fn i64_eq_panic_second_arg() {
    let wat = r#"
  (module
    (func (export "i64_eq") (result i32)
      i64.const 1
      i32.const 1
      i64.eq
    )
  )
"#;

    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");

    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), ())
            .unwrap()
    );
}

/// A function to test the i64.eq implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/5741d6c5172866174fde27c6b5447af757528d1a/i64.wast#L304)
#[test_log::test]
pub fn i64_eq() {
    let wat = String::from(WAT).replace("{{0}}", "eq");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0_i64, 0_i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1_i64, 1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1_i64, 1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x8000000000000000u64 as i64, 0x8000000000000000u64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x7fffffffffffffffu64 as i64, 0x7fffffffffffffffu64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1_i64, -1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1_i64, 0_i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0_i64, 1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x8000000000000000u64 as i64, 0_i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0_i64, 0x8000000000000000u64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x8000000000000000u64 as i64, -1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1_i64, 0x8000000000000000u64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x8000000000000000u64 as i64, 0x7fffffffffffffffu64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x7fffffffffffffffu64 as i64, 0x8000000000000000u64 as i64)
            )
            .unwrap()
    );
}

/// A function to test the i64.ne implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/5741d6c5172866174fde27c6b5447af757528d1a/i64.wast#L319)
#[test_log::test]
pub fn i64_ne() {
    let wat = String::from(WAT).replace("{{0}}", "ne");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0_i64, 0_i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1_i64, 1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1_i64, 1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x8000000000000000u64 as i64, 0x8000000000000000u64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x7fffffffffffffffu64 as i64, 0x7fffffffffffffffu64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1_i64, -1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1_i64, 0_i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0_i64, 1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x8000000000000000u64 as i64, 0_i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0_i64, 0x8000000000000000u64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x8000000000000000u64 as i64, -1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1_i64, 0x8000000000000000u64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x8000000000000000u64 as i64, 0x7fffffffffffffffu64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x7fffffffffffffffu64 as i64, 0x8000000000000000u64 as i64)
            )
            .unwrap()
    );
}

/// A function to test the i64.lt_s implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/5741d6c5172866174fde27c6b5447af757528d1a/i64.wast#L334)
#[test_log::test]
pub fn i64_lt_s() {
    let wat = String::from(WAT).replace("{{0}}", "lt_s");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0_i64, 0_i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1_i64, 1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1_i64, 1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x8000000000000000u64 as i64, 0x8000000000000000u64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x7fffffffffffffffu64 as i64, 0x7fffffffffffffffu64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1_i64, -1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1_i64, 0_i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0_i64, 1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x8000000000000000u64 as i64, 0_i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0_i64, 0x8000000000000000u64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x8000000000000000u64 as i64, -1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1_i64, 0x8000000000000000u64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x8000000000000000u64 as i64, 0x7fffffffffffffffu64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x7fffffffffffffffu64 as i64, 0x8000000000000000u64 as i64)
            )
            .unwrap()
    );
}

/// A function to test the i64.lt_u implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/5741d6c5172866174fde27c6b5447af757528d1a/i64.wast#L349)
#[test_log::test]
pub fn i64_lt_u() {
    let wat = String::from(WAT).replace("{{0}}", "lt_u");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0_i64, 0_i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1_i64, 1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1_i64, 1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x8000000000000000u64 as i64, 0x8000000000000000u64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x7fffffffffffffffu64 as i64, 0x7fffffffffffffffu64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1_i64, -1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1_i64, 0_i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0_i64, 1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x8000000000000000u64 as i64, 0_i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0_i64, 0x8000000000000000u64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x8000000000000000u64 as i64, -1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1_i64, 0x8000000000000000u64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x8000000000000000u64 as i64, 0x7fffffffffffffffu64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x7fffffffffffffffu64 as i64, 0x8000000000000000u64 as i64)
            )
            .unwrap()
    );
}

/// A function to test the i64.gt_s implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/5741d6c5172866174fde27c6b5447af757528d1a/i64.wast#L394)
#[test_log::test]
pub fn i64_gt_s() {
    let wat = String::from(WAT).replace("{{0}}", "gt_s");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0_i64, 0_i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1_i64, 1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1_i64, 1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x8000000000000000u64 as i64, 0x8000000000000000u64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x7fffffffffffffffu64 as i64, 0x7fffffffffffffffu64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1_i64, -1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1_i64, 0_i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0_i64, 1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x8000000000000000u64 as i64, 0_i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0_i64, 0x8000000000000000u64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x8000000000000000u64 as i64, -1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1_i64, 0x8000000000000000u64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x8000000000000000u64 as i64, 0x7fffffffffffffffu64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x7fffffffffffffffu64 as i64, 0x8000000000000000u64 as i64)
            )
            .unwrap()
    );
}

/// A function to test the i64.gt_u implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/5741d6c5172866174fde27c6b5447af757528d1a/i64.wast#L409)
#[test_log::test]
pub fn i64_gt_u() {
    let wat = String::from(WAT).replace("{{0}}", "gt_u");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0_i64, 0_i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1_i64, 1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1_i64, 1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x8000000000000000u64 as i64, 0x8000000000000000u64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x7fffffffffffffffu64 as i64, 0x7fffffffffffffffu64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1_i64, -1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1_i64, 0_i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0_i64, 1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x8000000000000000u64 as i64, 0_i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0_i64, 0x8000000000000000u64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x8000000000000000u64 as i64, -1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1_i64, 0x8000000000000000u64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x8000000000000000u64 as i64, 0x7fffffffffffffffu64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x7fffffffffffffffu64 as i64, 0x8000000000000000u64 as i64)
            )
            .unwrap()
    );
}

/// A function to test the i64.le_s implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/5741d6c5172866174fde27c6b5447af757528d1a/i64.wast#L364)
#[test_log::test]
pub fn i64_le_s() {
    let wat = String::from(WAT).replace("{{0}}", "le_s");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0_i64, 0_i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1_i64, 1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1_i64, 1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x8000000000000000u64 as i64, 0x8000000000000000u64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x7fffffffffffffffu64 as i64, 0x7fffffffffffffffu64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1_i64, -1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1_i64, 0_i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0_i64, 1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x8000000000000000u64 as i64, 0_i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0_i64, 0x8000000000000000u64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x8000000000000000u64 as i64, -1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1_i64, 0x8000000000000000u64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x8000000000000000u64 as i64, 0x7fffffffffffffffu64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x7fffffffffffffffu64 as i64, 0x8000000000000000u64 as i64)
            )
            .unwrap()
    );
}

/// A function to test the i64.le_u implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/5741d6c5172866174fde27c6b5447af757528d1a/i64.wast#L379)
#[test_log::test]
pub fn i64_le_u() {
    // todo!();
    let wat = String::from(WAT).replace("{{0}}", "le_u");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0_i64, 0_i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1_i64, 1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1_i64, 1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x8000000000000000u64 as i64, 0x8000000000000000u64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x7fffffffffffffffu64 as i64, 0x7fffffffffffffffu64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1_i64, -1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1_i64, 0_i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0_i64, 1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x8000000000000000u64 as i64, 0_i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0_i64, 0x8000000000000000u64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x8000000000000000u64 as i64, -1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1_i64, 0x8000000000000000u64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x8000000000000000u64 as i64, 0x7fffffffffffffffu64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x7fffffffffffffffu64 as i64, 0x8000000000000000u64 as i64)
            )
            .unwrap()
    );
}

/// A function to test the i64.ge_s implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/5741d6c5172866174fde27c6b5447af757528d1a/i64.wast#L424)
#[test_log::test]
pub fn i64_ge_s() {
    let wat = String::from(WAT).replace("{{0}}", "ge_s");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0_i64, 0_i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1_i64, 1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1_i64, 1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x8000000000000000u64 as i64, 0x8000000000000000u64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x7fffffffffffffffu64 as i64, 0x7fffffffffffffffu64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1_i64, -1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1_i64, 0_i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0_i64, 1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x8000000000000000u64 as i64, 0_i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0_i64, 0x8000000000000000u64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x8000000000000000u64 as i64, -1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1_i64, 0x8000000000000000u64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x8000000000000000u64 as i64, 0x7fffffffffffffffu64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x7fffffffffffffffu64 as i64, 0x8000000000000000u64 as i64)
            )
            .unwrap()
    );
}

/// A function to test the i64.ge_u implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/5741d6c5172866174fde27c6b5447af757528d1a/i64.wast#L439)
#[test_log::test]
pub fn i64_ge_u() {
    let wat = String::from(WAT).replace("{{0}}", "ge_u");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0_i64, 0_i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1_i64, 1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1_i64, 1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x8000000000000000u64 as i64, 0x8000000000000000u64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x7fffffffffffffffu64 as i64, 0x7fffffffffffffffu64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1_i64, -1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (1_i64, 0_i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0_i64, 1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x8000000000000000u64 as i64, 0_i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0_i64, 0x8000000000000000u64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x8000000000000000u64 as i64, -1_i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1_i64, 0x8000000000000000u64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x8000000000000000u64 as i64, 0x7fffffffffffffffu64 as i64)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x7fffffffffffffffu64 as i64, 0x8000000000000000u64 as i64)
            )
            .unwrap()
    );
}
