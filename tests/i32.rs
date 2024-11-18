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
      (func (export "i32_{{0}}") (param $x i32) (param $y i32) (result i32)
          local.get $x
          local.get $y
          i32.{{0}})
    )
"#;

/// A function to test the i32.add implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/5741d6c5172866174fde27c6b5447af757528d1a/i32.wast#L37)
#[test_log::test]
pub fn i32_add() {
    let wat = String::from(WAT).replace("{{0}}", "add");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        2,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (1, 1))
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (1, 0))
            .unwrap()
    );
    assert_eq!(
        -2,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (-1, -1))
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (-1, 1))
            .unwrap()
    );
    // Chaned the following value from the spec:
    // - 0x80000000 to -2147483648 = (0x80000000 as u32) as i32
    let i32_min = 0x80000000_u32 as i32;

    assert_eq!(
        i32_min,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x7fffffff, 1)
            )
            .unwrap()
    );
    assert_eq!(
        0x7fffffff,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (i32_min, -1)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (i32_min, i32_min)
            )
            .unwrap()
    );
    assert_eq!(
        0x40000000,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x3fffffff, 1)
            )
            .unwrap()
    );
}

/// A function to test the i32.sub implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/5741d6c5172866174fde27c6b5447af757528d1a/i32.wast#L46)
#[test_log::test]
pub fn i32_sub() {
    let wat = String::from(WAT).replace("{{0}}", "sub");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (1, 1))
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (1, 0))
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (-1, -1))
            .unwrap()
    );
    // Chaned the following value from the spec:
    // - 0x80000000 to -2147483648 = (0x80000000 as u32) as i32
    let i32_min = 0x80000000_u32 as i32;

    assert_eq!(
        i32_min,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x7fffffff, -1)
            )
            .unwrap()
    );
    assert_eq!(
        0x7fffffff,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (i32_min, 1))
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (i32_min, i32_min)
            )
            .unwrap()
    );
    assert_eq!(
        0x40000000,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x3fffffff, -1)
            )
            .unwrap()
    );
}

#[should_panic]
#[test_log::test]
pub fn i32_eqz_panic() {
    let wat = r#"
  (module
    (func (export "i32_eqz") (result i32)
      i64.const 1
      i32.eqz
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

/// A function to test the i32.eqz implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/5741d6c5172866174fde27c6b5447af757528d1a/i32.wast#L286)
#[test_log::test]
pub fn i32_eqz() {
    let wat = r#"
      (module
        (func (export "i32_eqz") (param $x i32) (result i32)
          local.get $x
          i32.eqz
        )
      )
    "#;

    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).expect("validation failed");

    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0)
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 1)
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0x80000000u32 as i32
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), 0x7fffffff)
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                0xffffffffu32 as i32
            )
            .unwrap()
    );
}

#[should_panic]
#[test_log::test]
pub fn i32_eq_panic_first_arg() {
    let wat = r#"
  (module
    (func (export "i32_eq") (result i32)
      i64.const 1
      i32.const 1
      i32.eq
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
pub fn i32_eq_panic_second_arg() {
    let wat = r#"
  (module
    (func (export "i32_eq") (result i32)
      i32.const 1
      i64.const 1
      i32.eq
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

/// A function to test the i32.eq implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/5741d6c5172866174fde27c6b5447af757528d1a/i32.wast#L292)
#[test_log::test]
pub fn i32_eq() {
    let wat = String::from(WAT).replace("{{0}}", "eq");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (0, 0))
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (1, 1))
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (-1, 1))
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x80000000u32 as i32, 0x80000000u32 as i32)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x7fffffff, 0x7fffffff)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (-1, -1))
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (1, 0))
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (0, 1))
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x80000000u32 as i32, 0)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0, 0x80000000u32 as i32)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x80000000u32 as i32, -1)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1, 0x80000000u32 as i32)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x80000000u32 as i32, 0x7fffffff)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x7fffffff, 0x80000000u32 as i32)
            )
            .unwrap()
    );
}

/// A function to test the i32.ne implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/5741d6c5172866174fde27c6b5447af757528d1a/i32.wast#L307)
#[test_log::test]
pub fn i32_ne() {
    let wat = String::from(WAT).replace("{{0}}", "ne");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (0, 0))
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (1, 1))
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (-1, 1))
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x80000000u32 as i32, 0x80000000u32 as i32)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x7fffffff, 0x7fffffff)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (-1, -1))
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (1, 0))
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (0, 1))
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x80000000u32 as i32, 0)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0, 0x80000000u32 as i32)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x80000000u32 as i32, -1)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1, 0x80000000u32 as i32)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x80000000u32 as i32, 0x7fffffff)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x7fffffff, 0x80000000u32 as i32)
            )
            .unwrap()
    );
}

/// A function to test the i32.lt_s implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/5741d6c5172866174fde27c6b5447af757528d1a/i32.wast#L322)
#[test_log::test]
pub fn i32_lt_s() {
    let wat = String::from(WAT).replace("{{0}}", "lt_s");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (0, 0))
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (1, 1))
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (-1, 1))
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x80000000u32 as i32, 0x80000000u32 as i32)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x7fffffff, 0x7fffffff)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (-1, -1))
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (1, 0))
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (0, 1))
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x80000000u32 as i32, 0)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0, 0x80000000u32 as i32)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x80000000u32 as i32, -1)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1, 0x80000000u32 as i32)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x80000000u32 as i32, 0x7fffffff)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x7fffffff, 0x80000000u32 as i32)
            )
            .unwrap()
    );
}

/// A function to test the i32.lt_u implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/5741d6c5172866174fde27c6b5447af757528d1a/i32.wast#L337)
#[test_log::test]
pub fn i32_lt_u() {
    let wat = String::from(WAT).replace("{{0}}", "lt_u");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (0, 0))
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (1, 1))
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (-1, 1))
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x80000000u32 as i32, 0x80000000u32 as i32)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x7fffffff, 0x7fffffff)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (-1, -1))
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (1, 0))
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (0, 1))
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x80000000u32 as i32, 0)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0, 0x80000000u32 as i32)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x80000000u32 as i32, -1)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1, 0x80000000u32 as i32)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x80000000u32 as i32, 0x7fffffff)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x7fffffff, 0x80000000u32 as i32)
            )
            .unwrap()
    );
}

/// A function to test the i32.gt_s implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/5741d6c5172866174fde27c6b5447af757528d1a/i32.wast#L382)
#[test_log::test]
pub fn i32_gt_s() {
    let wat = String::from(WAT).replace("{{0}}", "gt_s");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (0, 0))
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (1, 1))
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (-1, 1))
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x80000000u32 as i32, 0x80000000u32 as i32)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x7fffffff, 0x7fffffff)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (-1, -1))
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (1, 0))
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (0, 1))
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x80000000u32 as i32, 0)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0, 0x80000000u32 as i32)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x80000000u32 as i32, -1)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1, 0x80000000u32 as i32)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x80000000u32 as i32, 0x7fffffff)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x7fffffff, 0x80000000u32 as i32)
            )
            .unwrap()
    );
}

/// A function to test the i32.gt_u implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/5741d6c5172866174fde27c6b5447af757528d1a/i32.wast#L397)
#[test_log::test]
pub fn i32_gt_u() {
    let wat = String::from(WAT).replace("{{0}}", "gt_u");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (0, 0))
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (1, 1))
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (-1, 1))
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x80000000u32 as i32, 0x80000000u32 as i32)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x7fffffff, 0x7fffffff)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (-1, -1))
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (1, 0))
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (0, 1))
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x80000000u32 as i32, 0)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0, 0x80000000u32 as i32)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x80000000u32 as i32, -1)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1, 0x80000000u32 as i32)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x80000000u32 as i32, 0x7fffffff)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x7fffffff, 0x80000000u32 as i32)
            )
            .unwrap()
    );
}

/// A function to test the i32.le_s implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/5741d6c5172866174fde27c6b5447af757528d1a/i32.wast#L352)
#[test_log::test]
pub fn i32_le_s() {
    let wat = String::from(WAT).replace("{{0}}", "le_s");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (0, 0))
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (1, 1))
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (-1, 1))
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x80000000u32 as i32, 0x80000000u32 as i32)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x7fffffff, 0x7fffffff)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (-1, -1))
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (1, 0))
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (0, 1))
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x80000000u32 as i32, 0)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0, 0x80000000u32 as i32)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x80000000u32 as i32, -1)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1, 0x80000000u32 as i32)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x80000000u32 as i32, 0x7fffffff)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x7fffffff, 0x80000000u32 as i32)
            )
            .unwrap()
    );
}

/// A function to test the i32.le_u implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/5741d6c5172866174fde27c6b5447af757528d1a/i32.wast#L367)
#[test_log::test]
pub fn i32_le_u() {
    // todo!();
    let wat = String::from(WAT).replace("{{0}}", "le_u");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (0, 0))
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (1, 1))
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (-1, 1))
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x80000000u32 as i32, 0x80000000u32 as i32)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x7fffffff, 0x7fffffff)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (-1, -1))
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (1, 0))
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (0, 1))
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x80000000u32 as i32, 0)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0, 0x80000000u32 as i32)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x80000000u32 as i32, -1)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1, 0x80000000u32 as i32)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x80000000u32 as i32, 0x7fffffff)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x7fffffff, 0x80000000u32 as i32)
            )
            .unwrap()
    );
}

/// A function to test the i32.ge_s implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/5741d6c5172866174fde27c6b5447af757528d1a/i32.wast#L412)
#[test_log::test]
pub fn i32_ge_s() {
    let wat = String::from(WAT).replace("{{0}}", "ge_s");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (0, 0))
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (1, 1))
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (-1, 1))
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x80000000u32 as i32, 0x80000000u32 as i32)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x7fffffff, 0x7fffffff)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (-1, -1))
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (1, 0))
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (0, 1))
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x80000000u32 as i32, 0)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0, 0x80000000u32 as i32)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x80000000u32 as i32, -1)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1, 0x80000000u32 as i32)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x80000000u32 as i32, 0x7fffffff)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x7fffffff, 0x80000000u32 as i32)
            )
            .unwrap()
    );
}

/// A function to test the i32.ge_u implementation using the [WASM TestSuite](https://github.com/WebAssembly/testsuite/blob/5741d6c5172866174fde27c6b5447af757528d1a/i32.wast#L427)
#[test_log::test]
pub fn i32_ge_u() {
    let wat = String::from(WAT).replace("{{0}}", "ge_u");
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (0, 0))
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (1, 1))
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (-1, 1))
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x80000000u32 as i32, 0x80000000u32 as i32)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x7fffffff, 0x7fffffff)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (-1, -1))
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (1, 0))
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(&instance.get_function_by_index(0, 0).unwrap(), (0, 1))
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x80000000u32 as i32, 0)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0, 0x80000000u32 as i32)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x80000000u32 as i32, -1)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (-1, 0x80000000u32 as i32)
            )
            .unwrap()
    );
    assert_eq!(
        1,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x80000000u32 as i32, 0x7fffffff)
            )
            .unwrap()
    );
    assert_eq!(
        0,
        instance
            .invoke(
                &instance.get_function_by_index(0, 0).unwrap(),
                (0x7fffffff, 0x80000000u32 as i32)
            )
            .unwrap()
    );
}
