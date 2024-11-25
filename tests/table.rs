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
use wasm::value::{FuncRefForInteropValue, Ref};
use wasm::Error as GeneralError;
use wasm::{validate, RuntimeInstance};

macro_rules! get_func {
    ($instance:ident, $func_name:expr) => {
        &$instance.get_function_by_name("", $func_name).unwrap()
    };
}

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
        RuntimeInstance::new(&validation_info).expect("instantiation failed");
    });
}

#[test_log::test]
fn table_basic_2() {
    let w = r#"
    (module (table 0 funcref) (table 0 funcref))
    (module (table (import "spectest" "table") 0 funcref) (table 0 funcref))
"#
    .split("\n")
    .map(|el| el.trim())
    .filter(|el| !el.is_empty())
    .collect::<Vec<&str>>();

    w.iter().for_each(|wat| {
        let wasm_bytes = wat::parse_str(wat).unwrap();
        let validation_info = validate(&wasm_bytes).expect("validation failed");
        RuntimeInstance::new(&validation_info).expect("instantiation failed");
    });
}

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
        assert!(validation_info.err().unwrap() == GeneralError::UnknownTable);
    });
}

#[test_log::test]
fn table_size_minimum_must_not_be_greater_than_maximum() {
    let w = r#"
    (module (table 1 0 funcref))
    (module (table 0xffff_ffff 0 funcref))
"#
    .split("\n")
    .map(|el| el.trim())
    .filter(|el| !el.is_empty())
    .collect::<Vec<&str>>();

    w.iter().for_each(|wat| {
        let wasm_bytes = wat::parse_str(wat).unwrap();
        let validation_info = validate(&wasm_bytes);
        assert!(validation_info.err().unwrap() == GeneralError::InvalidLimit);
    });
}

#[test_log::test]
fn table_elem_test() {
    let w = r#"
    (module
        (table 2 funcref)
        (elem (i32.const 0) $f1 $f3)
        (func $f1 (result i32)
            i32.const 42)
        (func $f2 (result i32)
            i32.const 13)
        (func $f3 (result i64)
            i64.const 13)
        (func $f4 (result i32)
            i32.const 13)
    )"#;
    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let instance = RuntimeInstance::new(&validation_info).expect("instantiation failed");
    let table = &instance.store.tables[0];
    assert!(table.len() == 2);
    let wanted: [usize; 2] = [0, 2];
    table
        .elem
        .iter()
        .enumerate()
        .for_each(|(i, rref)| match *rref {
            wasm::value::Ref::Extern(_) => panic!(),
            wasm::value::Ref::Func(func_addr) => {
                assert!(func_addr.addr.is_some());
                assert!(wanted[i] == func_addr.addr.unwrap())
            }
        });
    // assert!(instance.store.tables)
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
    let mut i = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let get_funcref = get_func!(i, "get-funcref");
    let init = get_func!(i, "init");

    // assert the function at index 1 is a FuncRef and is NOT null
    {
        let funcref = i
            .invoke::<i32, FuncRefForInteropValue>(get_funcref, 1)
            .unwrap();

        let rref = funcref.get_ref();

        match rref {
            Ref::Func(funcaddr) => {
                assert!(!funcaddr.is_null())
            }
            _ => panic!("Expected a FuncRef"),
        }
    }

    // assert the function at index 2 is a FuncRef and is null
    {
        let funcref = i
            .invoke::<i32, FuncRefForInteropValue>(get_funcref, 2)
            .unwrap();

        let rref = funcref.get_ref();

        match rref {
            Ref::Func(funcaddr) => {
                assert!(funcaddr.is_null())
            }
            _ => panic!("Expected a FuncRef"),
        }
    }

    // set the function at index 2 the same as the one at index 1
    i.invoke::<(), ()>(init, ()).unwrap();
    // assert the function at index 2 is a FuncRef and is NOT null
    {
        let funcref = i
            .invoke::<i32, FuncRefForInteropValue>(get_funcref, 2)
            .unwrap();

        let rref = funcref.get_ref();

        match rref {
            Ref::Func(funcaddr) => {
                assert!(!funcaddr.is_null())
            }
            _ => panic!("Expected a FuncRef"),
        }
    }
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
//         // assert!(validation_info.err().unwrap() == GeneralError::InvalidLimit);
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