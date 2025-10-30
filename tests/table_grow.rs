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
use wasm::{validate, RuntimeError, RuntimeInstance, TrapError, DEFAULT_MODULE};

#[test_log::test]
fn table_grow_test() {
    let w = r#"
    (module
        (table $t 0 externref)
        (func (export "get") (param $i i32) (result externref) (table.get $t (local.get $i)))
        (func (export "set") (param $i i32) (param $r externref) (table.set $t (local.get $i) (local.get $r)))
        (func (export "grow") (param $sz i32) (param $init externref) (result i32)
            (table.grow $t (local.get $init) (local.get $sz))
        )
        (func (export "grow-abbrev") (param $sz i32) (param $init externref) (result i32)
            (table.grow (local.get $init) (local.get $sz))
        )
        (func (export "size") (result i32) (table.size $t))
    )
    "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let (mut i, module_addr) = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");

    let get = i.get_function_by_name(DEFAULT_MODULE, "get").unwrap();
    let set = i.get_function_by_name(DEFAULT_MODULE, "set").unwrap();
    let grow = i.get_function_by_name(DEFAULT_MODULE, "grow").unwrap();
    let grow_abbrev = i
        .get_function_by_name(DEFAULT_MODULE, "grow-abbrev")
        .unwrap();
    let size = i.get_function_by_name(DEFAULT_MODULE, "size").unwrap();

    assert_eq!(i.invoke_typed::<(), i32>(&size, ()), Ok(0));
    assert_eq!(
        i.invoke_typed::<(i32, RefExtern), ()>(&set, (0, RefExtern(Some(ExternAddr(2)))))
            .err(),
        Some(RuntimeError::Trap(
            TrapError::TableOrElementAccessOutOfBounds
        ))
    );
    assert_eq!(
        i.invoke_typed::<i32, RefExtern>(&get, 0).err(),
        Some(RuntimeError::Trap(
            TrapError::TableOrElementAccessOutOfBounds
        ))
    );

    assert_eq!(
        i.invoke_typed::<(i32, RefExtern), i32>(&grow, (1, RefExtern(None))),
        Ok(0)
    );
    assert_eq!(i.invoke_typed::<(), i32>(&size, ()), Ok(1));
    assert_eq!(
        i.invoke_typed::<i32, RefExtern>(&get, 0),
        Ok(RefExtern(None))
    );
    assert_eq!(
        i.invoke_typed::<(i32, RefExtern), ()>(&set, (0, RefExtern(Some(ExternAddr(2))))),
        Ok(())
    );
    assert_eq!(
        i.invoke_typed::<i32, RefExtern>(&get, 0),
        Ok(RefExtern(Some(ExternAddr(2))))
    );
    assert_eq!(
        i.invoke_typed::<(i32, RefExtern), ()>(&set, (1, RefExtern(Some(ExternAddr(2)))))
            .err(),
        Some(RuntimeError::Trap(
            TrapError::TableOrElementAccessOutOfBounds
        ))
    );
    assert_eq!(
        i.invoke_typed::<i32, RefExtern>(&get, 1).err(),
        Some(RuntimeError::Trap(
            TrapError::TableOrElementAccessOutOfBounds
        ))
    );

    assert_eq!(
        i.invoke_typed::<(i32, RefExtern), i32>(&grow_abbrev, (4, RefExtern(Some(ExternAddr(3))))),
        Ok(1)
    );
    assert_eq!(i.invoke_typed::<(), i32>(&size, ()), Ok(5));
    assert_eq!(
        i.invoke_typed::<i32, RefExtern>(&get, 0),
        Ok(RefExtern(Some(ExternAddr(2))))
    );
    assert_eq!(
        i.invoke_typed::<(i32, RefExtern), ()>(&set, (0, RefExtern(Some(ExternAddr(2))))),
        Ok(())
    );
    assert_eq!(
        i.invoke_typed::<i32, RefExtern>(&get, 0),
        Ok(RefExtern(Some(ExternAddr(2))))
    );
    assert_eq!(
        i.invoke_typed::<i32, RefExtern>(&get, 1),
        Ok(RefExtern(Some(ExternAddr(3))))
    );
    assert_eq!(
        i.invoke_typed::<i32, RefExtern>(&get, 4),
        Ok(RefExtern(Some(ExternAddr(3))))
    );
    assert_eq!(
        i.invoke_typed::<(i32, RefExtern), ()>(&set, (4, RefExtern(Some(ExternAddr(4))))),
        Ok(())
    );
    assert_eq!(
        i.invoke_typed::<i32, RefExtern>(&get, 4),
        Ok(RefExtern(Some(ExternAddr(4))))
    );
    assert_eq!(
        i.invoke_typed::<(i32, RefExtern), ()>(&set, (5, RefExtern(Some(ExternAddr(2)))))
            .err(),
        Some(RuntimeError::Trap(
            TrapError::TableOrElementAccessOutOfBounds
        ))
    );
    assert_eq!(
        i.invoke_typed::<i32, RefExtern>(&get, 5).err(),
        Some(RuntimeError::Trap(
            TrapError::TableOrElementAccessOutOfBounds
        ))
    );
}

// ... existing code ...

#[test_log::test]
fn table_grow_outside_i32_range() {
    let w = r#"
    (module
        (table $t 0x10 funcref)
        (elem declare func $f)
        (func $f (export "grow") (result i32)
            (table.grow $t (ref.func $f) (i32.const 0xffff_fff0))
        )
    )
    "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let (mut i, module_addr) = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");

    let grow = i.get_function_by_name(DEFAULT_MODULE, "grow").unwrap();
    assert_eq!(i.invoke_typed::<(), i32>(&grow, ()).unwrap(), -1);
}

#[test_log::test]
fn table_grow_unlimited() {
    let w = r#"
    (module
        (table $t 0 funcref)
        (func (export "grow") (param i32) (result i32)
            (table.grow $t (ref.null func) (local.get 0))
        )
    )
    "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let (mut i, module_addr) = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");

    let grow = i.get_function_by_name(DEFAULT_MODULE, "grow").unwrap();
    assert_eq!(i.invoke_typed(&grow, 0), Ok(0));
    assert_eq!(i.invoke_typed(&grow, 1), Ok(0));
    assert_eq!(i.invoke_typed(&grow, 0), Ok(1));
    assert_eq!(i.invoke_typed(&grow, 2), Ok(1));
    assert_eq!(i.invoke_typed(&grow, 800), Ok(3));
}

#[test_log::test]
fn table_grow_with_max() {
    let w = r#"
    (module
        (table $t 0 10 funcref)
        (func (export "grow") (param i32) (result i32)
            (table.grow $t (ref.null func) (local.get 0))
        )
    )
    "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let (mut i, module_addr) = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");

    let grow = i.get_function_by_name(DEFAULT_MODULE, "grow").unwrap();
    assert_eq!(i.invoke_typed(&grow, 0), Ok(0));
    assert_eq!(i.invoke_typed(&grow, 1), Ok(0));
    assert_eq!(i.invoke_typed(&grow, 1), Ok(1));
    assert_eq!(i.invoke_typed(&grow, 2), Ok(2));
    assert_eq!(i.invoke_typed(&grow, 6), Ok(4));
    assert_eq!(i.invoke_typed(&grow, 0), Ok(10));
    assert_eq!(i.invoke_typed(&grow, 1), Ok(-1));
    assert_eq!(i.invoke_typed(&grow, 0x10000), Ok(-1));
}

#[ignore = "control flow not yet implemented"]
#[test_log::test]
fn table_grow_check_null() {
    let w = r#"
    (module
        (table $t 10 funcref)
        (func (export "grow") (param i32) (result i32)
            (table.grow $t (ref.null func) (local.get 0))
        )
        (elem declare func 1)
        (func (export "check-table-null") (param i32 i32) (result funcref)
            (local funcref)
            (local.set 2 (ref.func 1))
            (block
                (loop
                    (local.set 2 (table.get $t (local.get 0)))
                    (br_if 1 (i32.eqz (ref.is_null (local.get 2))))
                    (br_if 1 (i32.ge_u (local.get 0) (local.get 1)))
                    (local.set 0 (i32.add (local.get 0) (i32.const 1)))
                    (br_if 0 (i32.le_u (local.get 0) (local.get 1)))
                )
            )
            (local.get 2)
        )
    )
    "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let (mut i, module_addr) = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");

    let grow = i.get_function_by_name(DEFAULT_MODULE, "grow").unwrap();
    let check_table_null = i
        .get_function_by_name(DEFAULT_MODULE, "check-table-null")
        .unwrap();

    assert_eq!(
        i.invoke_typed::<(i32, i32), RefExtern>(&check_table_null, (0, 9))
            .unwrap(),
        RefExtern(None)
    );
    assert_eq!(i.invoke_typed(&grow, 10), Ok(10));
    assert_eq!(
        i.invoke_typed::<(i32, i32), RefExtern>(&check_table_null, (0, 19))
            .unwrap(),
        RefExtern(None)
    );
}

#[test_log::test]
fn table_grow_with_exported_table_test() {
    // First module - Target with exported table
    let target_wat = r#"
    (module
        (table (export "table") 1 funcref)
        (func (export "grow") (result i32)
            (table.grow (ref.null func) (i32.const 1))
        )
    )
    "#;

    let wasm_bytes = wat::parse_str(target_wat).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let (mut target_instance, module_addr) =
        RuntimeInstance::new_with_default_module((), &validation_info)
            .expect("target instantiation failed");

    let grow = target_instance
        .get_function_by_name(DEFAULT_MODULE, "grow")
        .unwrap();
    assert_eq!(target_instance.invoke_typed(&grow, ()), Ok(1));
}

// #[test_log::test]
// fn table_grow_import_table_from_first_module() {
//     let import1_wat = r#"
//     (module
//         (table (export "table") (import "grown-table" "table") 2 funcref)
//         (func (export "grow") (result i32)
//             (table.grow (ref.null func) (i32.const 1))
//         )
//     )
//     "#;

//     let wasm_bytes = wat::parse_str(import1_wat).unwrap();
//     let validation_info = validate(&wasm_bytes).unwrap();
//     let mut import1_instance = RuntimeInstance::new(&validation_info).expect("import1 instantiation failed");

//     let grow = import1_instance.get_function_by_name(DEFAULT_MODULE, "grow").unwrap();
//     assert_eq!(import1_instance.invoke_typed( grow,  ()), Ok( 2));
// }

// #[ignore = "table exports not yet implemented"]
// #[test_log::test]
// fn table_grow_import_table_and_check_size() {
//     let import2_wat = r#"
//     (module
//         (import "grown-imported-table" "table" (table 3 funcref))
//         (func (export "size") (result i32)
//             (table.size)
//         )
//     )
//     "#;

//     let wasm_bytes = wat::parse_str(import2_wat).unwrap();
//     let validation_info = validate(&wasm_bytes).unwrap();
//     let mut import2_instance = RuntimeInstance::new(&validation_info).expect("import2 instantiation failed");

//     let size = import2_instance.get_function_by_name(DEFAULT_MODULE, "size").unwrap();
//     assert_eq!(import2_instance.invoke_typed( size,  ()), Ok( 3));
// }

// TODO: we can NOT run this test yet because ???
#[ignore = "table grow type errors"]
#[test_log::test]
fn table_grow_type_errors() {
    // Test cases for type errors
    let invalid_cases = [
        (
            r#"
            (module
                (table $t 0 funcref)
                (func $type-init-size-empty-vs-i32-funcref (result i32)
                    (table.grow $t)
                )
            )
            "#,
            "type mismatch",
        ),
        (
            r#"
            (module
                (table $t 0 funcref)
                (func $type-size-empty-vs-i32 (result i32)
                    (table.grow $t (ref.null func))
                )
            )
            "#,
            "type mismatch",
        ),
        (
            r#"
            (module
                (table $t 0 funcref)
                (func $type-init-empty-vs-funcref (result i32)
                    (table.grow $t (i32.const 1))
                )
            )
            "#,
            "type mismatch",
        ),
        // Add more invalid cases as needed
    ];

    for (wat, expected_error) in invalid_cases.iter() {
        let result = wat::parse_str(wat);
        assert!(result.is_err());
        assert!(result.err().unwrap().to_string().contains(expected_error));
    }
}
