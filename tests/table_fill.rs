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
use wasm::interop::RefExtern;
use wasm::value::ExternAddr;
use wasm::{validate, RuntimeError, RuntimeInstance, TrapError};

#[test_log::test]
fn table_fill_test() {
    let w = r#"
    (module
      (table $t 10 externref)

      (func (export "fill") (param $i i32) (param $r externref) (param $n i32)
        (table.fill $t (local.get $i) (local.get $r) (local.get $n))
      )

      (func (export "fill-abbrev") (param $i i32) (param $r externref) (param $n i32)
        (table.fill $t (local.get $i) (local.get $r) (local.get $n))
      )

      (func (export "get") (param $i i32) (result externref)
        (table.get $t (local.get $i))
      )
    )
    "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let (mut i, module) = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");

    let get = i
        .store
        .instance_export(module, "get")
        .unwrap()
        .as_func()
        .unwrap();
    let fill = i
        .store
        .instance_export(module, "fill")
        .unwrap()
        .as_func()
        .unwrap();
    let fill_abbrev = i
        .store
        .instance_export(module, "fill-abbrev")
        .unwrap()
        .as_func()
        .unwrap();

    assert_eq!(
        i.store.invoke_typed_without_fuel::<i32, RefExtern>(get, 1),
        Ok(RefExtern(None))
    );
    assert_eq!(
        i.store.invoke_typed_without_fuel::<i32, RefExtern>(get, 2),
        Ok(RefExtern(None))
    );
    assert_eq!(
        i.store.invoke_typed_without_fuel::<i32, RefExtern>(get, 3),
        Ok(RefExtern(None))
    );
    assert_eq!(
        i.store.invoke_typed_without_fuel::<i32, RefExtern>(get, 4),
        Ok(RefExtern(None))
    );
    assert_eq!(
        i.store.invoke_typed_without_fuel::<i32, RefExtern>(get, 5),
        Ok(RefExtern(None))
    );

    i.store
        .invoke_typed_without_fuel::<(i32, RefExtern, i32), ()>(
            fill,
            (2, RefExtern(Some(ExternAddr(1))), 3),
        )
        .unwrap();
    assert_eq!(
        i.store.invoke_typed_without_fuel::<i32, RefExtern>(get, 1),
        Ok(RefExtern(None))
    );
    assert_eq!(
        i.store
            .invoke_typed_without_fuel::<i32, RefExtern>(get, 2)
            .unwrap()
            .0
            .unwrap()
            .0,
        1
    );
    assert_eq!(
        i.store
            .invoke_typed_without_fuel::<i32, RefExtern>(get, 3)
            .unwrap()
            .0
            .unwrap()
            .0,
        1
    );
    assert_eq!(
        i.store
            .invoke_typed_without_fuel::<i32, RefExtern>(get, 4)
            .unwrap()
            .0
            .unwrap()
            .0,
        1
    );
    assert_eq!(
        i.store.invoke_typed_without_fuel::<i32, RefExtern>(get, 5),
        Ok(RefExtern(None))
    );

    i.store
        .invoke_typed_without_fuel::<(i32, RefExtern, i32), ()>(
            fill,
            (4, RefExtern(Some(ExternAddr(2))), 2),
        )
        .unwrap();

    assert_eq!(
        i.store
            .invoke_typed_without_fuel::<i32, RefExtern>(get, 3)
            .unwrap()
            .0
            .unwrap()
            .0,
        1
    );
    assert_eq!(
        i.store
            .invoke_typed_without_fuel::<i32, RefExtern>(get, 4)
            .unwrap()
            .0
            .unwrap()
            .0,
        2
    );
    assert_eq!(
        i.store
            .invoke_typed_without_fuel::<i32, RefExtern>(get, 5)
            .unwrap()
            .0
            .unwrap()
            .0,
        2
    );
    assert_eq!(
        i.store.invoke_typed_without_fuel::<i32, RefExtern>(get, 6),
        Ok(RefExtern(None))
    );

    i.store
        .invoke_typed_without_fuel::<(i32, RefExtern, i32), ()>(
            fill,
            (4, RefExtern(Some(ExternAddr(3))), 0),
        )
        .unwrap();

    assert_eq!(
        i.store
            .invoke_typed_without_fuel::<i32, RefExtern>(get, 3)
            .unwrap()
            .0
            .unwrap()
            .0,
        1
    );
    assert_eq!(
        i.store
            .invoke_typed_without_fuel::<i32, RefExtern>(get, 4)
            .unwrap()
            .0
            .unwrap()
            .0,
        2
    );
    assert_eq!(
        i.store
            .invoke_typed_without_fuel::<i32, RefExtern>(get, 5)
            .unwrap()
            .0
            .unwrap()
            .0,
        2
    );

    i.store
        .invoke_typed_without_fuel::<(i32, RefExtern, i32), ()>(
            fill,
            (8, RefExtern(Some(ExternAddr(4))), 2),
        )
        .unwrap();

    assert_eq!(
        i.store.invoke_typed_without_fuel::<i32, RefExtern>(get, 7),
        Ok(RefExtern(None))
    );
    assert_eq!(
        i.store
            .invoke_typed_without_fuel::<i32, RefExtern>(get, 8)
            .unwrap()
            .0
            .unwrap()
            .0,
        4
    );
    assert_eq!(
        i.store
            .invoke_typed_without_fuel::<i32, RefExtern>(get, 9)
            .unwrap()
            .0
            .unwrap()
            .0,
        4
    );

    i.store
        .invoke_typed_without_fuel::<(i32, RefExtern, i32), ()>(
            fill_abbrev,
            (9, RefExtern(None), 1),
        )
        .unwrap();
    assert_eq!(
        i.store
            .invoke_typed_without_fuel::<i32, RefExtern>(get, 8)
            .unwrap()
            .0
            .unwrap()
            .0,
        4
    );
    assert_eq!(
        i.store.invoke_typed_without_fuel::<i32, RefExtern>(get, 9),
        Ok(RefExtern(None))
    );

    i.store
        .invoke_typed_without_fuel::<(i32, RefExtern, i32), ()>(
            fill,
            (10, RefExtern(Some(ExternAddr(5))), 0),
        )
        .unwrap();
    assert_eq!(
        i.store.invoke_typed_without_fuel::<i32, RefExtern>(get, 9),
        Ok(RefExtern(None))
    );

    assert_eq!(
        i.store
            .invoke_typed_without_fuel::<(i32, RefExtern, i32), ()>(
                fill,
                (8, RefExtern(Some(ExternAddr(6))), 3)
            )
            .err(),
        Some(RuntimeError::Trap(
            TrapError::TableOrElementAccessOutOfBounds
        ))
    );

    assert_eq!(
        i.store.invoke_typed_without_fuel::<i32, RefExtern>(get, 7),
        Ok(RefExtern(None))
    );
    assert_eq!(
        i.store
            .invoke_typed_without_fuel::<i32, RefExtern>(get, 8)
            .unwrap()
            .0
            .unwrap()
            .0,
        4
    );
    assert_eq!(
        i.store.invoke_typed_without_fuel::<i32, RefExtern>(get, 9),
        Ok(RefExtern(None))
    );

    assert_eq!(
        i.store
            .invoke_typed_without_fuel::<(i32, RefExtern, i32), ()>(fill, (11, RefExtern(None), 0))
            .err(),
        Some(RuntimeError::Trap(
            TrapError::TableOrElementAccessOutOfBounds
        ))
    );

    assert_eq!(
        i.store
            .invoke_typed_without_fuel::<(i32, RefExtern, i32), ()>(fill, (11, RefExtern(None), 10))
            .err(),
        Some(RuntimeError::Trap(
            TrapError::TableOrElementAccessOutOfBounds
        ))
    );
}
