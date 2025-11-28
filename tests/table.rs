use wasm::interop::RefFunc;
use wasm::value::Ref;
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
use wasm::{validate, Store};
use wasm::{ExternVal, ValidationError};

#[test_log::test]
fn table_basic() {
    let w = r#"
    (module (table 0 funcref))
    (module (table 1 funcref))
    (module (table 0 0 funcref))
    (module (table 0 1 funcref))
    (module (table 1 256 funcref))
    (module (table 0 65536 funcref))
    (module (table 0 0xffff_ffff funcref))
"#
    .split("\n")
    .map(|el| el.trim())
    .filter(|el| !el.is_empty())
    .collect::<Vec<&str>>();

    w.iter().for_each(|wat| {
        let wasm_bytes = wat::parse_str(wat).unwrap();
        let validation_info = validate(&wasm_bytes).expect("validation failed");
        let mut store = Store::new(());
        store
            .module_instantiate_unchecked(&validation_info, Vec::new(), None)
            .unwrap();
    });
}

// #[test_log::test]
// fn table_basic_2() {
//     let w = r#"
//     (module (table 0 funcref) (table 0 funcref))
//     (module (table (import "spectest" "table") 0 funcref) (table 0 funcref))
// "#
//     .split("\n")
//     .map(|el| el.trim())
//     .filter(|el| !el.is_empty())
//     .collect::<Vec<&str>>();

//     w.iter().for_each(|wat| {
//         let wasm_bytes = wat::parse_str(wat).unwrap();
//         let validation_info = validate(&wasm_bytes).expect("validation failed");
//         RuntimeInstance::new(&validation_info)
//     });
// }

#[test_log::test]
fn unknown_table() {
    let w = r#"
    (module (elem (i32.const 0)))
    (module (elem (i32.const 0) $f) (func $f))
"#
    .split("\n")
    .map(|el| el.trim())
    .filter(|el| !el.is_empty())
    .collect::<Vec<&str>>();

    w.iter().for_each(|wat| {
        let wasm_bytes = wat::parse_str(wat).unwrap();
        let validation_info = validate(&wasm_bytes);
        assert_eq!(
            validation_info.err(),
            Some(ValidationError::InvalidTableIdx(0))
        );
    });
}

#[test_log::test]
fn table_size_minimum_must_not_be_greater_than_maximum() {
    {
        let module = "(module (table 1 0 funcref))";
        let wasm_bytes = wat::parse_str(module).unwrap();
        let validation_info = validate(&wasm_bytes);
        assert_eq!(
            validation_info.err(),
            Some(ValidationError::MalformedLimitsMinLargerThanMax { min: 1, max: 0 })
        );
    }

    {
        let module = "(module (table 0xffff_ffff 0 funcref))";
        let wasm_bytes = wat::parse_str(module).unwrap();
        let validation_info = validate(&wasm_bytes);
        assert_eq!(
            validation_info.err(),
            Some(ValidationError::MalformedLimitsMinLargerThanMax {
                min: 0xFFFF_FFFF,
                max: 0
            })
        );
    }
}

#[test_log::test]
fn table_elem_test() {
    let w = r#"
    (module
        (table (export "tab") 2 funcref)
        (elem (i32.const 0) $f1 $f3)
        (func $f1 (export "f1") (result i32)
            i32.const 42)
        (func $f2 (export "f2") (result i32)
            i32.const 13)
        (func $f3 (export "f3") (result i64)
            i64.const 13)
        (func $f4 (export "f4") (result i32)
            i32.const 13)
    )"#;
    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut store = Store::new(());
    let module = store
        .module_instantiate_unchecked(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;

    let f1 = store
        .instance_export_unchecked(module, "f1")
        .unwrap()
        .as_func()
        .unwrap();
    let f3 = store
        .instance_export_unchecked(module, "f3")
        .unwrap()
        .as_func()
        .unwrap();

    let Ok(ExternVal::Table(table)) = store.instance_export_unchecked(module, "tab") else {
        panic!("expected a table to be exported")
    };

    assert_eq!(store.table_read_unchecked(table, 0), Ok(Ref::Func(f1)));
    assert_eq!(store.table_read_unchecked(table, 1), Ok(Ref::Func(f3)));
}

#[test_log::test]
fn table_get_set_test() {
    let w = r#"
(module
    (table $t3 3 funcref)
    (elem (table $t3) (i32.const 1) func $dummy)
    (elem func $dummypassive)
    (func $dummypassive)
    (func $dummy)
    (func (export "init")
        (table.set $t3 (i32.const 2) (table.get $t3 (i32.const 1)))
    )
    (func $f3 (export "get-funcref") (param $i i32) (result funcref)
        (table.get $t3 (local.get $i))
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

    let get_funcref = store
        .instance_export_unchecked(module, "get-funcref")
        .unwrap()
        .as_func()
        .unwrap();
    let init = store
        .instance_export_unchecked(module, "init")
        .unwrap()
        .as_func()
        .unwrap();

    // assert the function at index 1 is a FuncRef and is NOT null
    {
        let funcref = store
            .invoke_typed_without_fuel_unchecked::<i32, RefFunc>(get_funcref, 1)
            .unwrap();

        assert!(funcref.0.is_some());
    }

    // assert the function at index 2 is a FuncRef and is null
    {
        let funcref = store
            .invoke_typed_without_fuel_unchecked::<i32, RefFunc>(get_funcref, 2)
            .unwrap();

        assert!(funcref.0.is_none());
    }

    // set the function at index 2 the same as the one at index 1
    store
        .invoke_typed_without_fuel_unchecked::<(), ()>(init, ())
        .unwrap();
    // assert the function at index 2 is a FuncRef and is NOT null
    {
        let funcref = store
            .invoke_typed_without_fuel_unchecked::<i32, RefFunc>(get_funcref, 2)
            .unwrap();

        assert!(funcref.0.is_some());
    }
}

#[test_log::test]
fn call_indirect_type_check() {
    let wat = r#"
    (module
    ;; duplicate same type for different ids to make sure types themselves are compared
    ;; during call_indirect, not type ids
    (type $type_1 (func (param i32) (result i32)))
    (type $type_2 (func (param i32) (result i32)))
    (type $type_3 (func (param i32) (result i32)))

    (func $add_one_func (type $type_1) (param $x i32) (result i32)
        local.get $x
        i32.const 1
        i32.add
    )

    (func $mul_two_func (type $type_2) (param $x i32) (result i32)
        local.get $x
        i32.const 2
        i32.mul
    )

    (table funcref (elem $add_one_func $mul_two_func))

    (func $call_function (param $value i32) (param $index i32) (result i32)
        local.get $value
        local.get $index
        call_indirect 0 (type $type_3)
    )

    (export "call_function" (func $call_function))
    )
    "#;
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let validation_info = validate(&wasm_bytes).expect("validation failed");
    let mut store = Store::new(());
    let module = store
        .module_instantiate_unchecked(&validation_info, Vec::new(), None)
        .unwrap()
        .module_addr;

    let call_fn = store
        .instance_export_unchecked(module, "call_function")
        .unwrap()
        .as_func()
        .unwrap();

    assert_eq!(
        4,
        store
            .invoke_typed_without_fuel_unchecked::<(i32, i32), i32>(call_fn, (3, 0))
            .unwrap()
    );
    assert_eq!(
        6,
        store
            .invoke_typed_without_fuel_unchecked::<(i32, i32), i32>(call_fn, (5, 0))
            .unwrap()
    );
    assert_eq!(
        6,
        store
            .invoke_typed_without_fuel_unchecked::<(i32, i32), i32>(call_fn, (3, 1))
            .unwrap()
    );
    assert_eq!(
        10,
        store
            .invoke_typed_without_fuel_unchecked::<(i32, i32), i32>(call_fn, (5, 1))
            .unwrap()
    );
}

// (assert_malformed
//   (module quote "(table 0x1_0000_0000 funcref)")
//   "i32 constant out of range"
// )
// (assert_malformed
//   (module quote "(table 0x1_0000_0000 0x1_0000_0000 funcref)")
//   "i32 constant out of range"
// )
// (assert_malformed
//   (module quote "(table 0 0x1_0000_0000 funcref)")
//   "i32 constant out of range"
// )

// ;; Duplicate table identifiers

// #[test_log::test]
// fn duplicate_table() {
//     let w = r#"
//     (module quote "(table $foo 1 funcref)" "(table $foo 1 funcref)")
// "#
//     .split("\n")
//     .map(|el| el.trim())
//     .filter(|el| !el.is_empty())
//     .collect::<Vec<&str>>();

//     w.iter().for_each(|wat| {
//         let wasm_bytes = wat::parse_str(wat).unwrap();
//         let validation_info = validate(&wasm_bytes);
//         // assert!(validation_info.err().unwrap() == ValidationError::InvalidLimit);
//     });
// }

// (assert_malformed (module quote
//   "(table $foo 1 funcref)"
//   "(table $foo 1 funcref)")
//   "duplicate table")
// (assert_malformed (module quote
//   "(import \"\" \"\" (table $foo 1 funcref))"
//   "(table $foo 1 funcref)")
//   "duplicate table")
// (assert_malformed (module quote
//   "(import \"\" \"\" (table $foo 1 funcref))"
//   "(import \"\" \"\" (table $foo 1 funcref))")
//   "duplicate table")
