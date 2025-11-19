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
    let mut i = RuntimeInstance::new(());
    let module = i
        .store
        .module_instantiate(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;

    // let get_funcref = i.store.instance_export(module, "get-funcref").unwrap().as_func().unwrap();
    // let init = i.store.instance_export(module, "init").unwrap().as_func().unwrap();
    let size_t0 = i
        .store
        .instance_export(module, "size-t0")
        .unwrap()
        .as_func()
        .unwrap();
    let size_t1 = i
        .store
        .instance_export(module, "size-t1")
        .unwrap()
        .as_func()
        .unwrap();
    let size_t2 = i
        .store
        .instance_export(module, "size-t2")
        .unwrap()
        .as_func()
        .unwrap();
    let size_t3 = i
        .store
        .instance_export(module, "size-t3")
        .unwrap()
        .as_func()
        .unwrap();
    let grow_t0 = i
        .store
        .instance_export(module, "grow-t0")
        .unwrap()
        .as_func()
        .unwrap();
    let grow_t1 = i
        .store
        .instance_export(module, "grow-t1")
        .unwrap()
        .as_func()
        .unwrap();
    let grow_t2 = i
        .store
        .instance_export(module, "grow-t2")
        .unwrap()
        .as_func()
        .unwrap();
    let grow_t3 = i
        .store
        .instance_export(module, "grow-t3")
        .unwrap()
        .as_func()
        .unwrap();

    assert_eq!(i.store.invoke_typed_without_fuel(size_t0, ()), Ok(0));
    assert_eq!(i.store.invoke_typed_without_fuel(size_t0, ()), Ok(0));
    assert_eq!(i.store.invoke_typed_without_fuel(grow_t0, 1), Ok(()));
    assert_eq!(i.store.invoke_typed_without_fuel(size_t0, ()), Ok(1));
    assert_eq!(i.store.invoke_typed_without_fuel(grow_t0, 4), Ok(()));
    assert_eq!(i.store.invoke_typed_without_fuel(size_t0, ()), Ok(5));
    assert_eq!(i.store.invoke_typed_without_fuel(grow_t0, 0), Ok(()));
    assert_eq!(i.store.invoke_typed_without_fuel(size_t0, ()), Ok(5));

    assert_eq!(i.store.invoke_typed_without_fuel(size_t1, ()), Ok(1));
    assert_eq!(i.store.invoke_typed_without_fuel(grow_t1, 1), Ok(()));
    assert_eq!(i.store.invoke_typed_without_fuel(size_t1, ()), Ok(2));
    assert_eq!(i.store.invoke_typed_without_fuel(grow_t1, 4), Ok(()));
    assert_eq!(i.store.invoke_typed_without_fuel(size_t1, ()), Ok(6));
    assert_eq!(i.store.invoke_typed_without_fuel(grow_t1, 0), Ok(()));
    assert_eq!(i.store.invoke_typed_without_fuel(size_t1, ()), Ok(6));

    assert_eq!(i.store.invoke_typed_without_fuel(size_t2, ()), Ok(0));
    assert_eq!(i.store.invoke_typed_without_fuel(grow_t2, 3), Ok(()));
    assert_eq!(i.store.invoke_typed_without_fuel(size_t2, ()), Ok(0));
    assert_eq!(i.store.invoke_typed_without_fuel(grow_t2, 1), Ok(()));
    assert_eq!(i.store.invoke_typed_without_fuel(size_t2, ()), Ok(1));
    assert_eq!(i.store.invoke_typed_without_fuel(grow_t2, 0), Ok(()));
    assert_eq!(i.store.invoke_typed_without_fuel(size_t2, ()), Ok(1));
    assert_eq!(i.store.invoke_typed_without_fuel(grow_t2, 4), Ok(()));
    assert_eq!(i.store.invoke_typed_without_fuel(size_t2, ()), Ok(1));
    assert_eq!(i.store.invoke_typed_without_fuel(grow_t2, 1), Ok(()));
    assert_eq!(i.store.invoke_typed_without_fuel(size_t2, ()), Ok(2));

    assert_eq!(i.store.invoke_typed_without_fuel(size_t3, ()), Ok(3));
    assert_eq!(i.store.invoke_typed_without_fuel(grow_t3, 1), Ok(()));
    assert_eq!(i.store.invoke_typed_without_fuel(size_t3, ()), Ok(4));
    assert_eq!(i.store.invoke_typed_without_fuel(grow_t3, 3), Ok(()));
    assert_eq!(i.store.invoke_typed_without_fuel(size_t3, ()), Ok(7));
    assert_eq!(i.store.invoke_typed_without_fuel(grow_t3, 0), Ok(()));
    assert_eq!(i.store.invoke_typed_without_fuel(size_t3, ()), Ok(7));
    assert_eq!(i.store.invoke_typed_without_fuel(grow_t3, 2), Ok(()));
    assert_eq!(i.store.invoke_typed_without_fuel(size_t3, ()), Ok(7));
    assert_eq!(i.store.invoke_typed_without_fuel(grow_t3, 1), Ok(()));
    assert_eq!(i.store.invoke_typed_without_fuel(size_t3, ()), Ok(8));
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
