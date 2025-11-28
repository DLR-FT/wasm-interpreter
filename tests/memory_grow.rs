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
use wasm::{validate, RuntimeError, Store, TrapError};

#[test_log::test]
fn memory_grow_test_1() {
    let w = r#"
    (module
        (memory 0)

        (func (export "load_at_zero") (result i32) (i32.load (i32.const 0)))
        (func (export "store_at_zero") (i32.store (i32.const 0) (i32.const 2)))

        (func (export "load_at_page_size") (result i32) (i32.load (i32.const 0x10000)))
        (func (export "store_at_page_size") (i32.store (i32.const 0x10000) (i32.const 3)))

        (func (export "grow") (param $sz i32) (result i32) (memory.grow (local.get $sz)))
        (func (export "size") (result i32) (memory.size))
    )
  "#;
    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut store = Store::new(());
    let module = store
        .module_instantiate_unchecked(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;

    let load_at_zero = store
        .instance_export_unchecked(module, "load_at_zero")
        .unwrap()
        .as_func()
        .unwrap();
    let store_at_zero = store
        .instance_export_unchecked(module, "store_at_zero")
        .unwrap()
        .as_func()
        .unwrap();
    let load_at_page_size = store
        .instance_export_unchecked(module, "load_at_page_size")
        .unwrap()
        .as_func()
        .unwrap();
    let store_at_page_size = store
        .instance_export_unchecked(module, "store_at_page_size")
        .unwrap()
        .as_func()
        .unwrap();
    let grow = store
        .instance_export_unchecked(module, "grow")
        .unwrap()
        .as_func()
        .unwrap();
    let size = store
        .instance_export_unchecked(module, "size")
        .unwrap()
        .as_func()
        .unwrap();

    // let x = store.invoke_typed_without_fuel(function_ref, params)
    assert_eq!(store.invoke_typed_without_fuel_unchecked(size, ()), Ok(0));
    assert_eq!(
        store
            .invoke_typed_without_fuel_unchecked::<(), ()>(store_at_zero, ())
            .err(),
        Some(RuntimeError::Trap(TrapError::MemoryOrDataAccessOutOfBounds))
    );
    assert_eq!(
        store
            .invoke_typed_without_fuel_unchecked::<(), i32>(load_at_zero, ())
            .err(),
        Some(RuntimeError::Trap(TrapError::MemoryOrDataAccessOutOfBounds))
    );

    assert_eq!(
        store
            .invoke_typed_without_fuel_unchecked::<(), ()>(store_at_page_size, ())
            .err(),
        Some(RuntimeError::Trap(TrapError::MemoryOrDataAccessOutOfBounds))
    );
    assert_eq!(
        store
            .invoke_typed_without_fuel_unchecked::<(), i32>(load_at_page_size, ())
            .err(),
        Some(RuntimeError::Trap(TrapError::MemoryOrDataAccessOutOfBounds))
    );
    assert_eq!(store.invoke_typed_without_fuel_unchecked(grow, 1), Ok(0));
    assert_eq!(store.invoke_typed_without_fuel_unchecked(size, ()), Ok(1));
    assert_eq!(
        store.invoke_typed_without_fuel_unchecked(load_at_zero, ()),
        Ok(0)
    );
    assert_eq!(
        store.invoke_typed_without_fuel_unchecked(store_at_zero, ()),
        Ok(())
    );
    assert_eq!(
        store.invoke_typed_without_fuel_unchecked(load_at_zero, ()),
        Ok(2)
    );
    assert_eq!(
        store
            .invoke_typed_without_fuel_unchecked::<(), ()>(store_at_page_size, ())
            .err(),
        Some(RuntimeError::Trap(TrapError::MemoryOrDataAccessOutOfBounds))
    );
    assert_eq!(
        store
            .invoke_typed_without_fuel_unchecked::<(), i32>(load_at_page_size, ())
            .err(),
        Some(RuntimeError::Trap(TrapError::MemoryOrDataAccessOutOfBounds))
    );
    assert_eq!(store.invoke_typed_without_fuel_unchecked(grow, 4), Ok(1));
    assert_eq!(store.invoke_typed_without_fuel_unchecked(size, ()), Ok(5));
    assert_eq!(
        store.invoke_typed_without_fuel_unchecked(load_at_zero, ()),
        Ok(2)
    );
    assert_eq!(
        store.invoke_typed_without_fuel_unchecked(store_at_zero, ()),
        Ok(())
    );
    assert_eq!(
        store.invoke_typed_without_fuel_unchecked(load_at_zero, ()),
        Ok(2)
    );
    assert_eq!(
        store.invoke_typed_without_fuel_unchecked(load_at_page_size, ()),
        Ok(0)
    );
    assert_eq!(
        store.invoke_typed_without_fuel_unchecked(store_at_page_size, ()),
        Ok(())
    );
    assert_eq!(
        store.invoke_typed_without_fuel_unchecked(load_at_page_size, ()),
        Ok(3)
    );
}

#[test_log::test]
#[cfg_attr(miri, ignore)] // test is too slow for miri
fn memory_grow_test_2() {
    let w = r#"
    (module
        (memory 0)
        (func (export "grow") (param i32) (result i32) (memory.grow (local.get 0)))
    )
  "#;
    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut store = Store::new(());
    let module = store
        .module_instantiate_unchecked(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;

    let grow = store
        .instance_export_unchecked(module, "grow")
        .unwrap()
        .as_func()
        .unwrap();

    assert_eq!(store.invoke_typed_without_fuel_unchecked(grow, 0), Ok(0));
    assert_eq!(store.invoke_typed_without_fuel_unchecked(grow, 1), Ok(0));
    assert_eq!(store.invoke_typed_without_fuel_unchecked(grow, 0), Ok(1));
    assert_eq!(store.invoke_typed_without_fuel_unchecked(grow, 2), Ok(1));
    assert_eq!(store.invoke_typed_without_fuel_unchecked(grow, 800), Ok(3));
    assert_eq!(
        store.invoke_typed_without_fuel_unchecked(grow, 0x10000),
        Ok(-1)
    );
    assert_eq!(
        store.invoke_typed_without_fuel_unchecked(grow, 64736),
        Ok(-1)
    );
    assert_eq!(store.invoke_typed_without_fuel_unchecked(grow, 1), Ok(803));
}

#[test_log::test]
#[cfg_attr(miri, ignore)] // test is too slow for miri
fn memory_grow_test_3() {
    let w = r#"
    (module
        (memory 0 10)
        (func (export "grow") (param i32) (result i32) (memory.grow (local.get 0)))
    )
  "#;
    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut store = Store::new(());
    let module = store
        .module_instantiate_unchecked(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;

    let grow = store
        .instance_export_unchecked(module, "grow")
        .unwrap()
        .as_func()
        .unwrap();

    assert_eq!(store.invoke_typed_without_fuel_unchecked(grow, 0), Ok(0));
    assert_eq!(store.invoke_typed_without_fuel_unchecked(grow, 1), Ok(0));
    assert_eq!(store.invoke_typed_without_fuel_unchecked(grow, 1), Ok(1));
    assert_eq!(store.invoke_typed_without_fuel_unchecked(grow, 2), Ok(2));
    assert_eq!(store.invoke_typed_without_fuel_unchecked(grow, 6), Ok(4));
    assert_eq!(store.invoke_typed_without_fuel_unchecked(grow, 0), Ok(10));
    assert_eq!(store.invoke_typed_without_fuel_unchecked(grow, 1), Ok(-1));
    assert_eq!(
        store.invoke_typed_without_fuel_unchecked(grow, 0x10000),
        Ok(-1)
    );
}
