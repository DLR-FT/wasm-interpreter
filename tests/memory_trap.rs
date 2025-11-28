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
fn memory_trap_1() {
    let w = r#"
(module
    (memory 1)

    (func $addr_limit (result i32)
      (i32.mul (memory.size) (i32.const 0x10000))
    )

    (func (export "store") (param $i i32) (param $v i32)
      (i32.store (i32.add (call $addr_limit) (local.get $i)) (local.get $v))
    )

    (func (export "load") (param $i i32) (result i32)
      (i32.load (i32.add (call $addr_limit) (local.get $i)))
    )

    (func (export "memory.grow") (param i32) (result i32)
      (memory.grow (local.get 0))
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

    let store_func = store
        .instance_export_unchecked(module, "store")
        .unwrap()
        .as_func()
        .unwrap();
    let load = store
        .instance_export_unchecked(module, "load")
        .unwrap()
        .as_func()
        .unwrap();
    let grow = store
        .instance_export_unchecked(module, "memory.grow")
        .unwrap()
        .as_func()
        .unwrap();

    assert_eq!(
        store.invoke_typed_without_fuel_unchecked(store_func, (-4, 42)),
        Ok(())
    );
    assert_eq!(store.invoke_typed_without_fuel_unchecked(load, -4), Ok(42));
    assert_eq!(
        store
            .invoke_typed_without_fuel_unchecked::<(i32, i32), ()>(store_func, (-3, 0x12345678))
            .err(),
        Some(RuntimeError::Trap(TrapError::MemoryOrDataAccessOutOfBounds))
    );
    assert_eq!(
        store
            .invoke_typed_without_fuel_unchecked::<i32, i32>(load, -3)
            .err(),
        Some(RuntimeError::Trap(TrapError::MemoryOrDataAccessOutOfBounds))
    );
    assert_eq!(
        store
            .invoke_typed_without_fuel_unchecked::<(i32, i32), ()>(store_func, (-2, 13))
            .err(),
        Some(RuntimeError::Trap(TrapError::MemoryOrDataAccessOutOfBounds))
    );
    assert_eq!(
        store
            .invoke_typed_without_fuel_unchecked::<i32, i32>(load, -2)
            .err(),
        Some(RuntimeError::Trap(TrapError::MemoryOrDataAccessOutOfBounds))
    );
    assert_eq!(
        store
            .invoke_typed_without_fuel_unchecked::<(i32, i32), ()>(store_func, (-1, 13))
            .err(),
        Some(RuntimeError::Trap(TrapError::MemoryOrDataAccessOutOfBounds))
    );
    assert_eq!(
        store
            .invoke_typed_without_fuel_unchecked::<i32, i32>(load, -1)
            .err(),
        Some(RuntimeError::Trap(TrapError::MemoryOrDataAccessOutOfBounds))
    );
    assert_eq!(
        store
            .invoke_typed_without_fuel_unchecked::<(i32, i32), ()>(store_func, (0, 13))
            .err(),
        Some(RuntimeError::Trap(TrapError::MemoryOrDataAccessOutOfBounds))
    );
    assert_eq!(
        store
            .invoke_typed_without_fuel_unchecked::<i32, i32>(load, 0)
            .err(),
        Some(RuntimeError::Trap(TrapError::MemoryOrDataAccessOutOfBounds))
    );
    assert_eq!(
        store
            .invoke_typed_without_fuel_unchecked::<(i32, i32), ()>(
                store_func,
                (0x80000000_u32 as i32, 13)
            )
            .err(),
        Some(RuntimeError::Trap(TrapError::MemoryOrDataAccessOutOfBounds))
    );
    assert_eq!(
        store
            .invoke_typed_without_fuel_unchecked::<i32, i32>(load, 0x80000000_u32 as i32)
            .err(),
        Some(RuntimeError::Trap(TrapError::MemoryOrDataAccessOutOfBounds))
    );
    assert_eq!(
        store.invoke_typed_without_fuel_unchecked(grow, 0x10001),
        Ok(-1)
    );
}
