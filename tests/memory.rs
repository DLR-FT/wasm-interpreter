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
use wasm::{validate, RuntimeInstance, ValidationError, DEFAULT_MODULE};

#[test_log::test]
fn memory_basic() {
    let w = r#"
(module (memory 0))
(module (memory 1))
(module (memory 0 0))
(module (memory 0 1))
(module (memory 1 256))
(module (memory 0 65536))
"#
    .split("\n")
    .map(|el| el.trim())
    .filter(|el| !el.is_empty())
    .collect::<Vec<&str>>();

    w.iter().for_each(|wat| {
        let wasm_bytes = wat::parse_str(wat).unwrap();
        let validation_info = validate(&wasm_bytes).expect("validation failed");
        RuntimeInstance::new_with_default_module((), &validation_info)
            .expect("instantiation failed");
    });
}

#[test_log::test]
fn memory_min_greater_than_max() {
    let w = r#"
      (module (memory 1 0))
    "#
    .split("\n")
    .map(|el| el.trim())
    .filter(|el| !el.is_empty())
    .collect::<Vec<&str>>();

    w.iter().for_each(|wat| {
        let wasm_bytes = wat::parse_str(wat).unwrap();
        let validation_info = validate(&wasm_bytes);
        assert_eq!(
            validation_info.err().unwrap(),
            ValidationError::MalformedLimitsMinLargerThanMax { min: 1, max: 0 }
        );
    });
}

#[test_log::test]
fn memory_size_must_be_at_most_4gib() {
    let w = r#"
    (module (memory 65537))
    (module (memory 2147483648))
    (module (memory 4294967295))
    (module (memory 0 65537))
    (module (memory 0 2147483648))
    (module (memory 0 4294967295))
        "#
    .split("\n")
    .map(|el| el.trim())
    .filter(|el| !el.is_empty())
    .collect::<Vec<&str>>();

    w.iter().for_each(|wat| {
        let wasm_bytes = wat::parse_str(wat).unwrap();
        let validation_info = validate(&wasm_bytes);
        assert_eq!(
            validation_info.err().unwrap(),
            ValidationError::MemoryTooLarge
        );
    });
}

macro_rules! get_func {
    ($instance:ident, $func_name:expr) => {
        &$instance
            .get_function_by_name(DEFAULT_MODULE, $func_name)
            .unwrap()
    };
}

macro_rules! assert_result {
    ($instance:expr, $func_name:expr, $arg:expr, $result:expr) => {
        assert_eq!($result, $instance.invoke_typed($func_name, $arg).unwrap());
    };
}

#[test_log::test]
fn i32_and_i64_loads() {
    // #region Wat
    let w = r#"
  (module
    (memory 1)
    (data (i32.const 0) "ABC\a7D") (data (i32.const 20) "WASM")

    ;; Data section
      (func (export "data") (result i32)
        (i32.and
          (i32.and
            (i32.and
              (i32.eq (i32.load8_u (i32.const 0)) (i32.const 65))
              (i32.eq (i32.load8_u (i32.const 3)) (i32.const 167))
            )
            (i32.and
              (i32.eq (i32.load8_u (i32.const 6)) (i32.const 0))
              (i32.eq (i32.load8_u (i32.const 19)) (i32.const 0))
            )
          )
          (i32.and
            (i32.and
              (i32.eq (i32.load8_u (i32.const 20)) (i32.const 87))
              (i32.eq (i32.load8_u (i32.const 23)) (i32.const 77))
            )
            (i32.and
              (i32.eq (i32.load8_u (i32.const 24)) (i32.const 0))
              (i32.eq (i32.load8_u (i32.const 1023)) (i32.const 0))
            )
          )
        )
      )

    ;; Memory cast
;;    (func (export "cast") (result f64)
;;      (i64.store (i32.const 8) (i64.const -12345))
;;      (if
;;        (f64.eq
;;          (f64.load (i32.const 8))
;;          (f64.reinterpret_i64 (i64.const -12345))
;;        )
;;        (then (return (f64.const 0)))
;;      )
;;      (i64.store align=1 (i32.const 9) (i64.const 0))
;;      (i32.store16 align=1 (i32.const 15) (i32.const 16453))
;;      (f64.load align=1 (i32.const 9))
;;    )

    (func (export "i32_load8_s") (param $i i32) (result i32)
      (i32.store8 (i32.const 8) (local.get $i))
      (i32.load8_s (i32.const 8))
    )
    (func (export "i32_load8_u") (param $i i32) (result i32)
      (i32.store8 (i32.const 8) (local.get $i))
      (i32.load8_u (i32.const 8))
    )
    (func (export "i32_load16_s") (param $i i32) (result i32)
      (i32.store16 (i32.const 8) (local.get $i))
      (i32.load16_s (i32.const 8))
    )
    (func (export "i32_load16_u") (param $i i32) (result i32)
      (i32.store16 (i32.const 8) (local.get $i))
      (i32.load16_u (i32.const 8))
    )
    (func (export "i64_load8_s") (param $i i64) (result i64)
      (i64.store8 (i32.const 8) (local.get $i))
      (i64.load8_s (i32.const 8))
    )
    (func (export "i64_load8_u") (param $i i64) (result i64)
      (i64.store8 (i32.const 8) (local.get $i))
      (i64.load8_u (i32.const 8))
    )
    (func (export "i64_load16_s") (param $i i64) (result i64)
      (i64.store16 (i32.const 8) (local.get $i))
      (i64.load16_s (i32.const 8))
    )
    (func (export "i64_load16_u") (param $i i64) (result i64)
      (i64.store16 (i32.const 8) (local.get $i))
      (i64.load16_u (i32.const 8))
    )
    (func (export "i64_load32_s") (param $i i64) (result i64)
      (i64.store32 (i32.const 8) (local.get $i))
      (i64.load32_s (i32.const 8))
    )
    (func (export "i64_load32_u") (param $i i64) (result i64)
      (i64.store32 (i32.const 8) (local.get $i))
      (i64.load32_u (i32.const 8))
    )
  )
      "#;
    // #endregion

    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut i = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");

    let i32_load8_s = get_func!(i, "i32_load8_s");
    let i32_load8_u = get_func!(i, "i32_load8_u");
    let i32_load16_s = get_func!(i, "i32_load16_s");
    let i32_load16_u = get_func!(i, "i32_load16_u");
    let i64_load8_s = get_func!(i, "i64_load8_s");
    let i64_load8_u = get_func!(i, "i64_load8_u");
    let i64_load16_s = get_func!(i, "i64_load16_s");
    let i64_load16_u = get_func!(i, "i64_load16_u");
    let i64_load32_s = get_func!(i, "i64_load32_s");
    let i64_load32_u = get_func!(i, "i64_load32_u");
    let data = get_func!(i, "data");
    // let cast = get_func!(i, "cast");

    // assert_result!(i, data, (), 1);
    assert_eq!(1, i.invoke_typed(data, ()).unwrap());
    // (assert_return (invoke "cast") (f64.const 42.0))

    assert_result!(i, i32_load8_s, -1, -1);
    assert_result!(i, i32_load8_s, -1, -1);
    assert_result!(i, i32_load8_u, -1, 255);
    assert_result!(i, i32_load16_s, -1, -1);
    assert_result!(i, i32_load16_u, -1, 65535);

    assert_result!(i, i32_load8_s, 100, 100);
    assert_result!(i, i32_load8_u, 200, 200);
    assert_result!(i, i32_load16_s, 20000, 20000);
    assert_result!(i, i32_load16_u, 40000, 40000);

    assert_result!(i, i32_load8_s, 0xfedc6543_u32, 0x43);
    assert_result!(i, i32_load8_s, 0x3456cdef, 0xffffffef_u32);
    assert_result!(i, i32_load8_u, 0xfedc6543_u32, 0x43);
    assert_result!(i, i32_load8_u, 0x3456cdef, 0xef);
    assert_result!(i, i32_load16_s, 0xfedc6543_u32, 0x6543);
    assert_result!(i, i32_load16_s, 0x3456cdef, 0xffffcdef_u32);
    assert_result!(i, i32_load16_u, 0xfedc6543_u32, 0x6543);
    assert_result!(i, i32_load16_u, 0x3456cdef, 0xcdef);

    assert_result!(i, i64_load8_s, -1_i64, -1_i64);
    assert_result!(i, i64_load8_u, -1_i64, 255_i64);
    assert_result!(i, i64_load16_s, -1_i64, -1_i64);
    assert_result!(i, i64_load16_u, -1_i64, 65535_i64);
    assert_result!(i, i64_load32_s, -1_i64, -1_i64);
    assert_result!(i, i64_load32_u, -1_i64, 4294967295_i64);

    assert_result!(i, i64_load8_s, 100_i64, 100_i64);
    assert_result!(i, i64_load8_u, 200_i64, 200_i64);
    assert_result!(i, i64_load16_s, 20000_i64, 20000_i64);
    assert_result!(i, i64_load16_u, 40000_i64, 40000_i64);
    assert_result!(i, i64_load32_s, 20000_i64, 20000_i64);
    assert_result!(i, i64_load32_u, 40000_i64, 40000_i64);

    assert_result!(i, i64_load8_s, 0xfedcba9856346543_u64, 0x43_i64);
    assert_result!(
        i,
        i64_load8_s,
        0x3456436598bacdef_u64,
        0xffffffffffffffef_u64
    );
    assert_result!(i, i64_load8_u, 0xfedcba9856346543_u64, 0x43_i64);
    assert_result!(i, i64_load8_u, 0x3456436598bacdef_u64, 0xef_i64);
    assert_result!(i, i64_load16_s, 0xfedcba9856346543_u64, 0x6543_i64);
    assert_result!(
        i,
        i64_load16_s,
        0x3456436598bacdef_u64,
        0xffffffffffffcdef_u64
    );
    assert_result!(i, i64_load16_u, 0xfedcba9856346543_u64, 0x6543_i64);
    assert_result!(i, i64_load16_u, 0x3456436598bacdef_u64, 0xcdef_i64);
    assert_result!(i, i64_load32_s, 0xfedcba9856346543_u64, 0x56346543_i64);
    assert_result!(
        i,
        i64_load32_s,
        0x3456436598bacdef_u64,
        0xffffffff98bacdef_u64
    );
    assert_result!(i, i64_load32_u, 0xfedcba9856346543_u64, 0x56346543_i64);
    assert_result!(i, i64_load32_u, 0x3456436598bacdef_u64, 0x98bacdef_i64);
}

#[test_log::test]
fn memory_test_exporting_rand_globals_doesnt_change_a_memory_s_semantics() {
    let w = r#"
    (module
      (memory (export "memory") 1 1)

      ;; These should not change the behavior of memory accesses.
      (global (export "__data_end") i32 (i32.const 10000))
      (global (export "__stack_top") i32 (i32.const 10000))
      (global (export "__heap_base") i32 (i32.const 10000))

      (func (export "load") (param i32) (result i32)
        (i32.load8_u (local.get 0))
      )
    )
  "#;
    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut i = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");

    let load = get_func!(i, "load");

    assert_result!(i, load, 0, 0);
    assert_result!(i, load, 10000, 0);
    assert_result!(i, load, 20000, 0);
    assert_result!(i, load, 30000, 0);
    assert_result!(i, load, 40000, 0);
    assert_result!(i, load, 50000, 0);
    assert_result!(i, load, 60000, 0);
    assert_result!(i, load, 65535, 0);
}
