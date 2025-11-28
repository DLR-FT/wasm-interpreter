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
use hexf::hexf32;
use wasm::{validate, Store};

#[test_log::test]
fn memory_redundancy() {
    let w = r#"
(module
  (memory 1 1)

  (func (export "zero_everything")
    (i32.store (i32.const 0) (i32.const 0))
    (i32.store (i32.const 4) (i32.const 0))
    (i32.store (i32.const 8) (i32.const 0))
    (i32.store (i32.const 12) (i32.const 0))
  )

  (func (export "test_store_to_load") (result i32)
    (i32.store (i32.const 8) (i32.const 0))
    (f32.store (i32.const 5) (f32.const -0.0))
    (i32.load (i32.const 8))
  )

  (func (export "test_redundant_load") (result i32)
    (local $t i32)
    (local $s i32)
    (local.set $t (i32.load (i32.const 8)))
    (i32.store (i32.const 5) (i32.const 0x80000000))
    (local.set $s (i32.load (i32.const 8)))
    (i32.add (local.get $t) (local.get $s))
  )

  (func (export "test_dead_store") (result f32)
    (local $t f32)
    (i32.store (i32.const 8) (i32.const 0x23232323))
    (local.set $t (f32.load (i32.const 11)))
    (i32.store (i32.const 8) (i32.const 0))
    (local.get $t)
  )

  ;; A function named "malloc" which implementations nonetheless shouldn't
  ;; assume behaves like C malloc.
  (func $malloc (export "malloc")
     (param $size i32)
     (result i32)
     (i32.const 16)
  )

  ;; Call malloc twice, but unlike C malloc, we don't get non-aliasing pointers.
  (func (export "malloc_aliasing")
     (result i32)
     (local $x i32)
     (local $y i32)
     (local.set $x (call $malloc (i32.const 4)))
     (local.set $y (call $malloc (i32.const 4)))
     (i32.store (local.get $x) (i32.const 42))
     (i32.store (local.get $y) (i32.const 43))
     (i32.load (local.get $x))
  )
)
  "#;
    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut store = Store::new(());
    let module = store
        .module_instantiate_unchecked(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;

    let test_store_to_load = store
        .instance_export_unchecked(module, "test_store_to_load")
        .unwrap()
        .as_func()
        .unwrap();
    let zero_everything = store
        .instance_export_unchecked(module, "zero_everything")
        .unwrap()
        .as_func()
        .unwrap();
    let test_redundant_load = store
        .instance_export_unchecked(module, "test_redundant_load")
        .unwrap()
        .as_func()
        .unwrap();
    let test_dead_store = store
        .instance_export_unchecked(module, "test_dead_store")
        .unwrap()
        .as_func()
        .unwrap();
    let malloc_aliasing = store
        .instance_export_unchecked(module, "malloc_aliasing")
        .unwrap()
        .as_func()
        .unwrap();

    assert_eq!(
        store.invoke_typed_without_fuel_unchecked(test_store_to_load, ()),
        Ok(0x00000080)
    );
    store
        .invoke_typed_without_fuel_unchecked::<(), ()>(zero_everything, ())
        .unwrap();
    assert_eq!(
        store.invoke_typed_without_fuel_unchecked(test_redundant_load, ()),
        Ok(0x00000080)
    );
    store
        .invoke_typed_without_fuel_unchecked::<(), ()>(zero_everything, ())
        .unwrap();
    assert_eq!(
        store.invoke_typed_without_fuel_unchecked(test_dead_store, ()),
        Ok(hexf32!("0x1.18p-144"))
    );
    store
        .invoke_typed_without_fuel_unchecked::<(), ()>(zero_everything, ())
        .unwrap();
    assert_eq!(
        store.invoke_typed_without_fuel_unchecked(malloc_aliasing, ()),
        Ok(43)
    );
}
