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

#[test_log::test]
fn table_size_test() {
    let w = r#"
(module
    (table $t0 0 externref)
    (table $t1 1 externref)
    (table $t2 0 2 externref)
    (table $t3 3 8 externref)

    (func (export "size-t0") (result i32) table.size)
    (func (export "size-t1") (result i32) (table.size $t1))
    (func (export "size-t2") (result i32) (table.size $t2))
    (func (export "size-t3") (result i32) (table.size $t3))

    (func (export "grow-t0") (param $sz i32)
      (drop (table.grow $t0 (ref.null extern) (local.get $sz)))
    )
    (func (export "grow-t1") (param $sz i32)
      (drop (table.grow $t1 (ref.null extern) (local.get $sz)))
    )
    (func (export "grow-t2") (param $sz i32)
      (drop (table.grow $t2 (ref.null extern) (local.get $sz)))
    )
    (func (export "grow-t3") (param $sz i32)
      (drop (table.grow $t3 (ref.null extern) (local.get $sz)))
    )
)
    "#;
    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let (mut i, _default_module) = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");

    // let get_funcref = i.get_function_by_name(DEFAULT_MODULE, "get-funcref").unwrap();
    // let init = i.get_function_by_name(DEFAULT_MODULE, "init").unwrap();
    let size_t0 = i.get_function_by_name(DEFAULT_MODULE, "size-t0").unwrap();
    let size_t1 = i.get_function_by_name(DEFAULT_MODULE, "size-t1").unwrap();
    let size_t2 = i.get_function_by_name(DEFAULT_MODULE, "size-t2").unwrap();
    let size_t3 = i.get_function_by_name(DEFAULT_MODULE, "size-t3").unwrap();
    let grow_t0 = i.get_function_by_name(DEFAULT_MODULE, "grow-t0").unwrap();
    let grow_t1 = i.get_function_by_name(DEFAULT_MODULE, "grow-t1").unwrap();
    let grow_t2 = i.get_function_by_name(DEFAULT_MODULE, "grow-t2").unwrap();
    let grow_t3 = i.get_function_by_name(DEFAULT_MODULE, "grow-t3").unwrap();

    assert_eq!(i.invoke_typed(size_t0, ()), Ok(0));
    assert_eq!(i.invoke_typed(size_t0, ()), Ok(0));
    assert_eq!(i.invoke_typed(grow_t0, 1), Ok(()));
    assert_eq!(i.invoke_typed(size_t0, ()), Ok(1));
    assert_eq!(i.invoke_typed(grow_t0, 4), Ok(()));
    assert_eq!(i.invoke_typed(size_t0, ()), Ok(5));
    assert_eq!(i.invoke_typed(grow_t0, 0), Ok(()));
    assert_eq!(i.invoke_typed(size_t0, ()), Ok(5));

    assert_eq!(i.invoke_typed(size_t1, ()), Ok(1));
    assert_eq!(i.invoke_typed(grow_t1, 1), Ok(()));
    assert_eq!(i.invoke_typed(size_t1, ()), Ok(2));
    assert_eq!(i.invoke_typed(grow_t1, 4), Ok(()));
    assert_eq!(i.invoke_typed(size_t1, ()), Ok(6));
    assert_eq!(i.invoke_typed(grow_t1, 0), Ok(()));
    assert_eq!(i.invoke_typed(size_t1, ()), Ok(6));

    assert_eq!(i.invoke_typed(size_t2, ()), Ok(0));
    assert_eq!(i.invoke_typed(grow_t2, 3), Ok(()));
    assert_eq!(i.invoke_typed(size_t2, ()), Ok(0));
    assert_eq!(i.invoke_typed(grow_t2, 1), Ok(()));
    assert_eq!(i.invoke_typed(size_t2, ()), Ok(1));
    assert_eq!(i.invoke_typed(grow_t2, 0), Ok(()));
    assert_eq!(i.invoke_typed(size_t2, ()), Ok(1));
    assert_eq!(i.invoke_typed(grow_t2, 4), Ok(()));
    assert_eq!(i.invoke_typed(size_t2, ()), Ok(1));
    assert_eq!(i.invoke_typed(grow_t2, 1), Ok(()));
    assert_eq!(i.invoke_typed(size_t2, ()), Ok(2));

    assert_eq!(i.invoke_typed(size_t3, ()), Ok(3));
    assert_eq!(i.invoke_typed(grow_t3, 1), Ok(()));
    assert_eq!(i.invoke_typed(size_t3, ()), Ok(4));
    assert_eq!(i.invoke_typed(grow_t3, 3), Ok(()));
    assert_eq!(i.invoke_typed(size_t3, ()), Ok(7));
    assert_eq!(i.invoke_typed(grow_t3, 0), Ok(()));
    assert_eq!(i.invoke_typed(size_t3, ()), Ok(7));
    assert_eq!(i.invoke_typed(grow_t3, 2), Ok(()));
    assert_eq!(i.invoke_typed(size_t3, ()), Ok(7));
    assert_eq!(i.invoke_typed(grow_t3, 1), Ok(()));
    assert_eq!(i.invoke_typed(size_t3, ()), Ok(8));
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
