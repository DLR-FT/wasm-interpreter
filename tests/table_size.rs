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

use wasm::{validate, RuntimeInstance, DEFAULT_MODULE};

macro_rules! get_func {
    ($instance:ident, $func_name:expr) => {
        &$instance
            .get_function_by_name(DEFAULT_MODULE, $func_name)
            .unwrap()
    };
}

macro_rules! assert_result {
    ($instance:expr, $func:expr, $arg:expr, $result:expr) => {
        assert_eq!($result, $instance.invoke_typed($func, $arg).unwrap());
    };
}

#[test_log::test]
fn table_size_test() {
    let w = r#"
(module
    (table $t0 0 funcref)
    (table $t1 1 funcref)
    (table $t2 0 2 funcref)
    (table $t3 3 8 funcref)
  
    (func (export "size-t0") (result i32) table.size)
    (func (export "size-t1") (result i32) (table.size $t1))
    (func (export "size-t2") (result i32) (table.size $t2))
    (func (export "size-t3") (result i32) (table.size $t3))
  
    (func (export "grow-t0") (param $sz i32)
      (drop (table.grow $t0 (ref.null func) (local.get $sz)))
    )
    (func (export "grow-t1") (param $sz i32)
      (drop (table.grow $t1 (ref.null func) (local.get $sz)))
    )
    (func (export "grow-t2") (param $sz i32)
      (drop (table.grow $t2 (ref.null func) (local.get $sz)))
    )
    (func (export "grow-t3") (param $sz i32)
      (drop (table.grow $t3 (ref.null func) (local.get $sz)))
    )
)
    "#;
    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut i =
        RuntimeInstance::new_with_default_module(&validation_info).expect("instantiation failed");

    // let get_funcref = get_func!(i, "get-funcref");
    // let init = get_func!(i, "init");
    let size_t0 = get_func!(i, "size-t0");
    let size_t1 = get_func!(i, "size-t1");
    let size_t2 = get_func!(i, "size-t2");
    let size_t3 = get_func!(i, "size-t3");
    let grow_t0 = get_func!(i, "grow-t0");
    let grow_t1 = get_func!(i, "grow-t1");
    let grow_t2 = get_func!(i, "grow-t2");
    let grow_t3 = get_func!(i, "grow-t3");

    assert_result!(i, size_t0, (), 0);
    assert_result!(i, grow_t0, 1, ());
    assert_result!(i, size_t0, (), 1);
    assert_result!(i, grow_t0, 4, ());
    assert_result!(i, size_t0, (), 5);
    assert_result!(i, grow_t0, 0, ());
    assert_result!(i, size_t0, (), 5);

    assert_result!(i, size_t1, (), 1);
    assert_result!(i, grow_t1, 1, ());
    assert_result!(i, size_t1, (), 2);
    assert_result!(i, grow_t1, 4, ());
    assert_result!(i, size_t1, (), 6);
    assert_result!(i, grow_t1, 0, ());
    assert_result!(i, size_t1, (), 6);

    assert_result!(i, size_t2, (), 0);
    assert_result!(i, grow_t2, 3, ());
    assert_result!(i, size_t2, (), 0);
    assert_result!(i, grow_t2, 1, ());
    assert_result!(i, size_t2, (), 1);
    assert_result!(i, grow_t2, 0, ());
    assert_result!(i, size_t2, (), 1);
    assert_result!(i, grow_t2, 4, ());
    assert_result!(i, size_t2, (), 1);
    assert_result!(i, grow_t2, 1, ());
    assert_result!(i, size_t2, (), 2);

    assert_result!(i, size_t3, (), 3);
    assert_result!(i, grow_t3, 1, ());
    assert_result!(i, size_t3, (), 4);
    assert_result!(i, grow_t3, 3, ());
    assert_result!(i, size_t3, (), 7);
    assert_result!(i, grow_t3, 0, ());
    assert_result!(i, size_t3, (), 7);
    assert_result!(i, grow_t3, 2, ());
    assert_result!(i, size_t3, (), 7);
    assert_result!(i, grow_t3, 1, ());
    assert_result!(i, size_t3, (), 8);
}

//   ;; Type errors

//   (assert_invalid
//     (module
//       (table $t 1 externref)
//       (func $type-result-i32-vs-empty
//         (table.size $t)
//       )
//     )
//     "type mismatch"
//   )
//   (assert_invalid
//     (module
//       (table $t 1 externref)
//       (func $type-result-i32-vs-f32 (result f32)
//         (table.size $t)
//       )
//     )
//     "type mismatch"
//   )
