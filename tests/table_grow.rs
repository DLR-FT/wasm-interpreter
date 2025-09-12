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
use wasm::value::{FuncAddr, FuncRefForInteropValue, Ref};
use wasm::{validate, RuntimeError, RuntimeInstance, DEFAULT_MODULE};

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
fn table_grow_test() {
    let w = r#"
    (module
        (table $t 0 funcref)
        (func (export "get") (param $i i32) (result funcref) (table.get $t (local.get $i)))
        (func (export "set") (param $i i32) (param $r funcref) (table.set $t (local.get $i) (local.get $r)))
        (func (export "grow") (param $sz i32) (param $init funcref) (result i32)
            (table.grow $t (local.get $init) (local.get $sz))
        )
        (func (export "grow-abbrev") (param $sz i32) (param $init funcref) (result i32)
            (table.grow (local.get $init) (local.get $sz))
        )
        (func (export "size") (result i32) (table.size $t))
    )
    "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut i = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");

    let get = get_func!(i, "get");
    let set = get_func!(i, "set");
    let grow = get_func!(i, "grow");
    let grow_abbrev = get_func!(i, "grow-abbrev");
    let size = get_func!(i, "size");

    assert!(i.invoke_typed::<(), i32>(size, ()).unwrap() == 0);
    assert!(
        i.invoke_typed::<(i32, FuncRefForInteropValue), ()>(
            set,
            (
                0,
                FuncRefForInteropValue::new(Ref::Func(FuncAddr::new(Some(2)))).unwrap()
            )
        )
        .err()
        .unwrap()
            == RuntimeError::TableOrElementAccessOutOfBounds
    );
    assert!(
        i.invoke_typed::<i32, FuncRefForInteropValue>(get, 0)
            .err()
            .unwrap()
            == RuntimeError::TableOrElementAccessOutOfBounds
    );

    assert!(
        i.invoke_typed::<(i32, FuncRefForInteropValue), i32>(
            grow,
            (
                1,
                FuncRefForInteropValue::new(Ref::Func(FuncAddr::null())).unwrap()
            )
        )
        .unwrap()
            == 0
    );
    assert!(i.invoke_typed::<(), i32>(size, ()).unwrap() == 1);
    assert!(i
        .invoke_typed::<i32, FuncRefForInteropValue>(get, 0)
        .unwrap()
        .get_ref()
        .is_null());
    assert!(i
        .invoke_typed::<(i32, FuncRefForInteropValue), ()>(
            set,
            (
                0,
                FuncRefForInteropValue::new(Ref::Func(FuncAddr::new(Some(2)))).unwrap()
            )
        )
        .is_ok());
    assert!(
        i.invoke_typed::<i32, FuncRefForInteropValue>(get, 0)
            .unwrap()
            == FuncRefForInteropValue::new(Ref::Func(FuncAddr::new(Some(2)))).unwrap()
    );
    assert!(
        i.invoke_typed::<(i32, FuncRefForInteropValue), ()>(
            set,
            (
                1,
                FuncRefForInteropValue::new(Ref::Func(FuncAddr::new(Some(2)))).unwrap()
            )
        )
        .err()
        .unwrap()
            == RuntimeError::TableOrElementAccessOutOfBounds
    );
    assert!(
        i.invoke_typed::<i32, FuncRefForInteropValue>(get, 1)
            .err()
            .unwrap()
            == RuntimeError::TableOrElementAccessOutOfBounds
    );

    assert!(
        i.invoke_typed::<(i32, FuncRefForInteropValue), i32>(
            grow_abbrev,
            (
                4,
                FuncRefForInteropValue::new(Ref::Func(FuncAddr::new(Some(3)))).unwrap()
            )
        )
        .unwrap()
            == 1
    );
    assert!(i.invoke_typed::<(), i32>(size, ()).unwrap() == 5);
    assert!(
        i.invoke_typed::<i32, FuncRefForInteropValue>(get, 0)
            .unwrap()
            == FuncRefForInteropValue::new(Ref::Func(FuncAddr::new(Some(2)))).unwrap()
    );
    assert!(i
        .invoke_typed::<(i32, FuncRefForInteropValue), ()>(
            set,
            (
                0,
                FuncRefForInteropValue::new(Ref::Func(FuncAddr::new(Some(2)))).unwrap()
            )
        )
        .is_ok());
    assert!(
        i.invoke_typed::<i32, FuncRefForInteropValue>(get, 0)
            .unwrap()
            == FuncRefForInteropValue::new(Ref::Func(FuncAddr::new(Some(2)))).unwrap()
    );
    assert!(
        i.invoke_typed::<i32, FuncRefForInteropValue>(get, 1)
            .unwrap()
            == FuncRefForInteropValue::new(Ref::Func(FuncAddr::new(Some(3)))).unwrap()
    );
    assert!(
        i.invoke_typed::<i32, FuncRefForInteropValue>(get, 4)
            .unwrap()
            == FuncRefForInteropValue::new(Ref::Func(FuncAddr::new(Some(3)))).unwrap()
    );
    assert!(i
        .invoke_typed::<(i32, FuncRefForInteropValue), ()>(
            set,
            (
                4,
                FuncRefForInteropValue::new(Ref::Func(FuncAddr::new(Some(4)))).unwrap()
            )
        )
        .is_ok());
    assert!(
        i.invoke_typed::<i32, FuncRefForInteropValue>(get, 4)
            .unwrap()
            == FuncRefForInteropValue::new(Ref::Func(FuncAddr::new(Some(4)))).unwrap()
    );
    assert!(
        i.invoke_typed::<(i32, FuncRefForInteropValue), ()>(
            set,
            (
                5,
                FuncRefForInteropValue::new(Ref::Func(FuncAddr::new(Some(2)))).unwrap()
            )
        )
        .err()
        .unwrap()
            == RuntimeError::TableOrElementAccessOutOfBounds
    );
    assert!(
        i.invoke_typed::<i32, FuncRefForInteropValue>(get, 5)
            .err()
            .unwrap()
            == RuntimeError::TableOrElementAccessOutOfBounds
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
    let mut i = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");

    let grow = get_func!(i, "grow");
    assert_eq!(i.invoke_typed::<(), i32>(grow, ()).unwrap(), -1);
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
    let mut i = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");

    let grow = get_func!(i, "grow");
    assert_result!(i, grow, 0, 0);
    assert_result!(i, grow, 1, 0);
    assert_result!(i, grow, 0, 1);
    assert_result!(i, grow, 2, 1);
    assert_result!(i, grow, 800, 3);
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
    let mut i = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");

    let grow = get_func!(i, "grow");
    assert_result!(i, grow, 0, 0);
    assert_result!(i, grow, 1, 0);
    assert_result!(i, grow, 1, 1);
    assert_result!(i, grow, 2, 2);
    assert_result!(i, grow, 6, 4);
    assert_result!(i, grow, 0, 10);
    assert_result!(i, grow, 1, -1);
    assert_result!(i, grow, 0x10000, -1);
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
    let mut i = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");

    let grow = get_func!(i, "grow");
    let check_table_null = get_func!(i, "check-table-null");

    assert_eq!(
        i.invoke_typed::<(i32, i32), FuncRefForInteropValue>(check_table_null, (0, 9))
            .unwrap(),
        FuncRefForInteropValue::new(Ref::Func(FuncAddr::null())).unwrap()
    );
    assert_result!(i, grow, 10, 10);
    assert_eq!(
        i.invoke_typed::<(i32, i32), FuncRefForInteropValue>(check_table_null, (0, 19))
            .unwrap(),
        FuncRefForInteropValue::new(Ref::Func(FuncAddr::null())).unwrap()
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
    let mut target_instance = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("target instantiation failed");

    let grow = get_func!(target_instance, "grow");
    assert_result!(target_instance, grow, (), 1);
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

//     let grow = get_func!(import1_instance, "grow");
//     assert_result!(import1_instance, grow, (), 2);
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

//     let size = get_func!(import2_instance, "size");
//     assert_result!(import2_instance, size, (), 3);
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
