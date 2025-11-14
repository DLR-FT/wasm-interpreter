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

use wasm::{
    interop::{RefExtern, RefFunc},
    validate,
    value::ExternAddr,
    RuntimeError, RuntimeInstance, TrapError,
};

#[test_log::test]
fn table_funcref_test() {
    let w = r#"
(module
  (table $t2 2 externref)
  (table $t3 3 funcref)
  (elem (table $t3) (i32.const 1) func $dummy)
  (func $dummy)

  (func (export "init") (param $r externref)
    (table.set $t2 (i32.const 1) (local.get $r))
    (table.set $t3 (i32.const 2) (table.get $t3 (i32.const 1)))
  )

  (func (export "get-externref") (param $i i32) (result externref)
    (table.get (local.get $i))
  )
  (func $f3 (export "get-funcref") (param $i i32) (result funcref)
    (table.get $t3 (local.get $i))
  )

  (func (export "is_null-funcref") (param $i i32) (result i32)
    (ref.is_null (call $f3 (local.get $i)))
  )
)
    "#;
    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let (mut i, module) = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");

    let init = i
        .store
        .instance_export(module, "init")
        .unwrap()
        .as_func()
        .unwrap();
    let get_externref = i
        .store
        .instance_export(module, "get-externref")
        .unwrap()
        .as_func()
        .unwrap();
    let get_funcref = i
        .store
        .instance_export(module, "get-funcref")
        .unwrap()
        .as_func()
        .unwrap();
    let is_null_funcref = i
        .store
        .instance_export(module, "is_null-funcref")
        .unwrap()
        .as_func()
        .unwrap();

    i.invoke_typed::<RefExtern, ()>(init, RefExtern(Some(ExternAddr(1))))
        .unwrap();

    assert_eq!(i.invoke_typed(get_externref, 0), Ok(RefExtern(None)));
    assert_eq!(
        i.invoke_typed(get_externref, 1),
        Ok(RefExtern(Some(ExternAddr(1))))
    );
    assert_eq!(i.invoke_typed(get_funcref, 0), Ok(RefFunc(None)));
    assert_eq!(i.invoke_typed(is_null_funcref, 1), Ok(0));
    assert_eq!(i.invoke_typed(is_null_funcref, 2), Ok(0));

    assert_eq!(
        i.invoke_typed::<i32, RefFunc>(get_externref, 2).err(),
        Some(RuntimeError::Trap(
            TrapError::TableOrElementAccessOutOfBounds
        ))
    );
    assert_eq!(
        i.invoke_typed::<i32, RefFunc>(get_funcref, 3).err(),
        Some(RuntimeError::Trap(
            TrapError::TableOrElementAccessOutOfBounds
        ))
    );
    assert_eq!(
        i.invoke_typed::<i32, RefFunc>(get_externref, -1).err(),
        Some(RuntimeError::Trap(
            TrapError::TableOrElementAccessOutOfBounds
        ))
    );
    assert_eq!(
        i.invoke_typed::<i32, RefFunc>(get_funcref, -1).err(),
        Some(RuntimeError::Trap(
            TrapError::TableOrElementAccessOutOfBounds
        ))
    );
}

#[test_log::test]
fn table_type_error_test() {
    let invalid_modules = vec![
        r#"(module (table $t 10 funcref) (func $type-index-empty-vs-i32 (result funcref) (table.get $t)))"#,
        r#"(module (table $t 10 funcref) (func $type-index-f32-vs-i32 (result funcref) (table.get $t (f32.const 1))))"#,
        r#"(module (table $t 10 funcref) (func $type-result-funcref-vs-empty (table.get $t (i32.const 0))))"#,
        r#"(module (table $t 10 funcref) (func $type-result-funcref-vs-funcref (result externref) (table.get $t (i32.const 1))))"#,
        r#"(module (table $t1 1 funcref) (table $t2 1 externref) (func $type-result-externref-vs-funcref-multi (result funcref) (table.get $t2 (i32.const 0))))"#,
    ];

    for module in invalid_modules {
        let wasm_bytes = wat::parse_str(module).unwrap();
        let result = validate(&wasm_bytes);
        assert!(
            result.is_err(),
            "Result `{result:?}` was expected to be `Err`, but it is not."
        );
    }
}
