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

use wasm::{validate, Error, RuntimeError, RuntimeInstance};

macro_rules! get_func {
    ($instance:ident, $func_name:expr) => {
        &$instance.get_function_by_name("", $func_name).unwrap()
    };
}

#[test_log::test]
fn table_init_1_test() {
    let w = r#"
    (module
        (type (func (result i32)))
        (func (export "ef0") (result i32) (i32.const 0))
        (func (export "ef1") (result i32) (i32.const 1))
        (func (export "ef2") (result i32) (i32.const 2))
        (func (export "ef3") (result i32) (i32.const 3))
        (func (export "ef4") (result i32) (i32.const 4))
        (table $t0 30 30 funcref)
        (table $t1 30 30 funcref)
        (elem (table $t0) (i32.const 2) func 3 1 4 1)
        (elem funcref
            (ref.func 2) (ref.func 7) (ref.func 1) (ref.func 8))
        (elem (table $t0) (i32.const 12) func 7 5 2 3 6)
        (elem funcref
            (ref.func 5) (ref.func 9) (ref.func 2) (ref.func 7) (ref.func 6))
        (func (result i32) (i32.const 5))  ;; index 5
        (func (result i32) (i32.const 6))
        (func (result i32) (i32.const 7))
        (func (result i32) (i32.const 8))
        (func (result i32) (i32.const 9))  ;; index 9
        (func (export "test")
            (table.init $t0 1 (i32.const 7) (i32.const 0) (i32.const 4)))
        (func (export "check") (param i32) (result i32)
            (call_indirect $t0 (type 0) (local.get 0)))
    )
    "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut i = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let test = get_func!(i, "test");
    let check = get_func!(i, "check");

    i.invoke::<(), ()>(test, ()).unwrap();

    assert!(i.invoke::<i32, i32>(check, 0).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 1).err().unwrap() == RuntimeError::UninitializedElement);
    assert_eq!(3, i.invoke(check, 2).unwrap());
    assert_eq!(1, i.invoke(check, 3).unwrap());
    assert_eq!(4, i.invoke(check, 4).unwrap());
    assert_eq!(1, i.invoke(check, 5).unwrap());
    assert!(i.invoke::<i32, i32>(check, 6).err().unwrap() == RuntimeError::UninitializedElement);
    assert_eq!(2, i.invoke(check, 7).unwrap());
    assert_eq!(7, i.invoke(check, 8).unwrap());
    assert_eq!(1, i.invoke(check, 9).unwrap());
    assert_eq!(8, i.invoke(check, 10).unwrap());
    assert!(i.invoke::<i32, i32>(check, 11).err().unwrap() == RuntimeError::UninitializedElement);
    assert_eq!(7, i.invoke(check, 12).unwrap());
    assert_eq!(5, i.invoke(check, 13).unwrap());
    assert_eq!(2, i.invoke(check, 14).unwrap());
    assert_eq!(3, i.invoke(check, 15).unwrap());
    assert_eq!(6, i.invoke(check, 16).unwrap());
    assert!(i.invoke::<i32, i32>(check, 17).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 18).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 19).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 20).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 21).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 22).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 23).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 24).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 25).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 26).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 27).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 28).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 29).err().unwrap() == RuntimeError::UninitializedElement);
}

#[test_log::test]
fn table_init_2_test() {
    let w = r#"
(module
    (type (func (result i32)))  ;; type #0
    (func (export "ef0") (result i32) (i32.const 0))
    (func (export "ef1") (result i32) (i32.const 1))
    (func (export "ef2") (result i32) (i32.const 2))
    (func (export "ef3") (result i32) (i32.const 3))
    (func (export "ef4") (result i32) (i32.const 4))
    (table $t0 30 30 funcref)
    (table $t1 30 30 funcref)
    (elem (table $t0) (i32.const 2) func 3 1 4 1)
    (elem funcref
        (ref.func 2) (ref.func 7) (ref.func 1) (ref.func 8)
    )
    (elem (table $t0) (i32.const 12) func 7 5 2 3 6)
    (elem funcref
        (ref.func 5) (ref.func 9) (ref.func 2) (ref.func 7) (ref.func 6)
    )
    (func (result i32) (i32.const 5))  ;; index 5
    (func (result i32) (i32.const 6))
    (func (result i32) (i32.const 7))
    (func (result i32) (i32.const 8))
    (func (result i32) (i32.const 9))  ;; index 9
    (func (export "test")
        (table.init $t0 3 (i32.const 15) (i32.const 1) (i32.const 3))
    )
    (func (export "check") (param i32) (result i32)
        (call_indirect $t0 (type 0) (local.get 0))
    )
)
    "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut i = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let test = get_func!(i, "test");
    let check = get_func!(i, "check");

    i.invoke::<(), ()>(test, ()).unwrap();

    assert!(i.invoke::<i32, i32>(check, 0).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 1).err().unwrap() == RuntimeError::UninitializedElement);
    assert_eq!(3, i.invoke(check, 2).unwrap());
    assert_eq!(1, i.invoke(check, 3).unwrap());
    assert_eq!(4, i.invoke(check, 4).unwrap());
    assert_eq!(1, i.invoke(check, 5).unwrap());
    assert!(i.invoke::<i32, i32>(check, 6).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 7).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 8).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 9).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 10).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 11).err().unwrap() == RuntimeError::UninitializedElement);
    assert_eq!(7, i.invoke(check, 12).unwrap());
    assert_eq!(5, i.invoke(check, 13).unwrap());
    assert_eq!(2, i.invoke(check, 14).unwrap());
    assert_eq!(9, i.invoke(check, 15).unwrap());
    assert_eq!(2, i.invoke(check, 16).unwrap());
    assert_eq!(7, i.invoke(check, 17).unwrap());
    assert!(i.invoke::<i32, i32>(check, 18).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 19).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 20).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 21).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 22).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 23).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 24).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 25).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 26).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 27).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 28).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 29).err().unwrap() == RuntimeError::UninitializedElement);
}

#[test_log::test]
fn table_init_3_test() {
    let w = r#"
(module
  (type (func (result i32)))  ;; type #0
  (func (export "ef0") (result i32) (i32.const 0))    ;; index 0
  (func (export "ef1") (result i32) (i32.const 1))
  (func (export "ef2") (result i32) (i32.const 2))
  (func (export "ef3") (result i32) (i32.const 3))
  (func (export "ef4") (result i32) (i32.const 4))    ;; index 4
  (table $t0 30 30 funcref)
  (table $t1 30 30 funcref)
  (elem (table $t0) (i32.const 2) func 3 1 4 1)
  (elem funcref
    (ref.func 2) (ref.func 7) (ref.func 1) (ref.func 8))
  (elem (table $t0) (i32.const 12) func 7 5 2 3 6)
  (elem funcref
    (ref.func 5) (ref.func 9) (ref.func 2) (ref.func 7) (ref.func 6))
  (func (result i32) (i32.const 5))  ;; index 5
  (func (result i32) (i32.const 6))
  (func (result i32) (i32.const 7))
  (func (result i32) (i32.const 8))
  (func (result i32) (i32.const 9))  ;; index 9
  (func (export "test")
    (table.init $t0 1 (i32.const 7) (i32.const 0) (i32.const 4))
         (elem.drop 1)
         (table.init $t0 3 (i32.const 15) (i32.const 1) (i32.const 3))
         (elem.drop 3)
         (table.copy $t0 0 (i32.const 20) (i32.const 15) (i32.const 5))
         (table.copy $t0 0 (i32.const 21) (i32.const 29) (i32.const 1))
         (table.copy $t0 0 (i32.const 24) (i32.const 10) (i32.const 1))
         (table.copy $t0 0 (i32.const 13) (i32.const 11) (i32.const 4))
         (table.copy $t0 0 (i32.const 19) (i32.const 20) (i32.const 5)))
  (func (export "check") (param i32) (result i32)
    (call_indirect $t0 (type 0) (local.get 0)))
)
    "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut i = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let test = get_func!(i, "test");
    let check = get_func!(i, "check");

    i.invoke::<(), ()>(test, ()).unwrap();

    assert!(i.invoke::<i32, i32>(check, 0).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 1).err().unwrap() == RuntimeError::UninitializedElement);
    assert_eq!(3, i.invoke(check, 2).unwrap());
    assert_eq!(1, i.invoke(check, 3).unwrap());
    assert_eq!(4, i.invoke(check, 4).unwrap());
    assert_eq!(1, i.invoke(check, 5).unwrap());
    assert!(i.invoke::<i32, i32>(check, 6).err().unwrap() == RuntimeError::UninitializedElement);
    assert_eq!(2, i.invoke(check, 7).unwrap());
    assert_eq!(7, i.invoke(check, 8).unwrap());
    assert_eq!(1, i.invoke(check, 9).unwrap());
    assert_eq!(8, i.invoke(check, 10).unwrap());
    assert!(i.invoke::<i32, i32>(check, 11).err().unwrap() == RuntimeError::UninitializedElement);
    assert_eq!(7, i.invoke(check, 12).unwrap());
    assert!(i.invoke::<i32, i32>(check, 13).err().unwrap() == RuntimeError::UninitializedElement);
    assert_eq!(7, i.invoke(check, 14).unwrap());
    assert_eq!(5, i.invoke(check, 15).unwrap());
    assert_eq!(2, i.invoke(check, 16).unwrap());
    assert_eq!(7, i.invoke(check, 17).unwrap());
    assert!(i.invoke::<i32, i32>(check, 18).err().unwrap() == RuntimeError::UninitializedElement);
    assert_eq!(9, i.invoke(check, 19).unwrap());
    assert!(i.invoke::<i32, i32>(check, 20).err().unwrap() == RuntimeError::UninitializedElement);
    assert_eq!(7, i.invoke(check, 21).unwrap());
    assert!(i.invoke::<i32, i32>(check, 22).err().unwrap() == RuntimeError::UninitializedElement);
    assert_eq!(8, i.invoke(check, 23).unwrap());
    assert_eq!(8, i.invoke(check, 24).unwrap());
    assert!(i.invoke::<i32, i32>(check, 25).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 26).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 27).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 28).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 29).err().unwrap() == RuntimeError::UninitializedElement);
}

#[test_log::test]
fn table_init_4_test() {
    let w = r#"
(module
  (type (func (result i32)))  ;; type #0
  (func (export "ef0") (result i32) (i32.const 0))    ;; index 0
  (func (export "ef1") (result i32) (i32.const 1))
  (func (export "ef2") (result i32) (i32.const 2))
  (func (export "ef3") (result i32) (i32.const 3))
  (func (export "ef4") (result i32) (i32.const 4))    ;; index 4
  (table $t0 30 30 funcref)
  (table $t1 30 30 funcref)
  (elem (table $t1) (i32.const 2) func 3 1 4 1)
  (elem funcref
    (ref.func 2) (ref.func 7) (ref.func 1) (ref.func 8))
  (elem (table $t1) (i32.const 12) func 7 5 2 3 6)
  (elem funcref
    (ref.func 5) (ref.func 9) (ref.func 2) (ref.func 7) (ref.func 6))
  (func (result i32) (i32.const 5))  ;; index 5
  (func (result i32) (i32.const 6))
  (func (result i32) (i32.const 7))
  (func (result i32) (i32.const 8))
  (func (result i32) (i32.const 9))  ;; index 9
  (func (export "test")
    (table.init $t1 3 (i32.const 15) (i32.const 1) (i32.const 3)))
  (func (export "check") (param i32) (result i32)
    (call_indirect $t1 (type 0) (local.get 0)))
)
    "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut i = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let test = get_func!(i, "test");
    let check = get_func!(i, "check");

    i.invoke::<(), ()>(test, ()).unwrap();

    println!("{:#?}", i.store.tables[1]);

    assert!(i.invoke::<i32, i32>(check, 0).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 1).err().unwrap() == RuntimeError::UninitializedElement);
    assert_eq!(3, i.invoke(check, 2).unwrap());
    assert_eq!(1, i.invoke(check, 3).unwrap());
    assert_eq!(4, i.invoke(check, 4).unwrap());
    assert_eq!(1, i.invoke(check, 5).unwrap());
    assert!(i.invoke::<i32, i32>(check, 6).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 7).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 8).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 9).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 10).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 11).err().unwrap() == RuntimeError::UninitializedElement);
    assert_eq!(7, i.invoke(check, 12).unwrap());
    assert_eq!(5, i.invoke(check, 13).unwrap());
    assert_eq!(2, i.invoke(check, 14).unwrap());
    assert_eq!(9, i.invoke(check, 15).unwrap());
    assert_eq!(2, i.invoke(check, 16).unwrap());
    assert_eq!(7, i.invoke(check, 17).unwrap());
    assert!(i.invoke::<i32, i32>(check, 18).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 19).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 20).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 21).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 22).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 23).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 24).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 25).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 26).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 27).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 28).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 29).err().unwrap() == RuntimeError::UninitializedElement);
}

#[test_log::test]
fn table_init_5_test() {
    let w = r#"
(module
  (type (func (result i32)))  ;; type #0
  (func (export "ef0") (result i32) (i32.const 0))    ;; index 0
  (func (export "ef1") (result i32) (i32.const 1))
  (func (export "ef2") (result i32) (i32.const 2))
  (func (export "ef3") (result i32) (i32.const 3))
  (func (export "ef4") (result i32) (i32.const 4))    ;; index 4
  (table $t0 30 30 funcref)
  (table $t1 30 30 funcref)
  (elem (table $t1) (i32.const 2) func 3 1 4 1)
  (elem funcref
    (ref.func 2) (ref.func 7) (ref.func 1) (ref.func 8))
  (elem (table $t1) (i32.const 12) func 7 5 2 3 6)
  (elem funcref
    (ref.func 5) (ref.func 9) (ref.func 2) (ref.func 7) (ref.func 6))
  (func (result i32) (i32.const 5))  ;; index 5
  (func (result i32) (i32.const 6))
  (func (result i32) (i32.const 7))
  (func (result i32) (i32.const 8))
  (func (result i32) (i32.const 9))  ;; index 9
  (func (export "test")
    (table.init $t1 1 (i32.const 7) (i32.const 0) (i32.const 4))
         (elem.drop 1)
         (table.init $t1 3 (i32.const 15) (i32.const 1) (i32.const 3))
         (elem.drop 3)
         (table.copy $t1 1 (i32.const 20) (i32.const 15) (i32.const 5))
         (table.copy $t1 1 (i32.const 21) (i32.const 29) (i32.const 1))
         (table.copy $t1 1 (i32.const 24) (i32.const 10) (i32.const 1))
         (table.copy $t1 1 (i32.const 13) (i32.const 11) (i32.const 4))
         (table.copy $t1 1 (i32.const 19) (i32.const 20) (i32.const 5)))
  (func (export "check") (param i32) (result i32)
    (call_indirect $t1 (type 0) (local.get 0)))
)
    "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut i = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let test = get_func!(i, "test");
    let check = get_func!(i, "check");

    i.invoke::<(), ()>(test, ()).unwrap();

    assert!(i.invoke::<i32, i32>(check, 0).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 1).err().unwrap() == RuntimeError::UninitializedElement);
    assert_eq!(3, i.invoke(check, 2).unwrap());
    assert_eq!(1, i.invoke(check, 3).unwrap());
    assert_eq!(4, i.invoke(check, 4).unwrap());
    assert_eq!(1, i.invoke(check, 5).unwrap());
    assert!(i.invoke::<i32, i32>(check, 6).err().unwrap() == RuntimeError::UninitializedElement);
    assert_eq!(2, i.invoke(check, 7).unwrap());
    assert_eq!(7, i.invoke(check, 8).unwrap());
    assert_eq!(1, i.invoke(check, 9).unwrap());
    assert_eq!(8, i.invoke(check, 10).unwrap());
    assert!(i.invoke::<i32, i32>(check, 11).err().unwrap() == RuntimeError::UninitializedElement);
    assert_eq!(7, i.invoke(check, 12).unwrap());
    assert!(i.invoke::<i32, i32>(check, 13).err().unwrap() == RuntimeError::UninitializedElement);
    assert_eq!(7, i.invoke(check, 14).unwrap());
    assert_eq!(5, i.invoke(check, 15).unwrap());
    assert_eq!(2, i.invoke(check, 16).unwrap());
    assert_eq!(7, i.invoke(check, 17).unwrap());
    assert!(i.invoke::<i32, i32>(check, 18).err().unwrap() == RuntimeError::UninitializedElement);
    assert_eq!(9, i.invoke(check, 19).unwrap());
    assert!(i.invoke::<i32, i32>(check, 20).err().unwrap() == RuntimeError::UninitializedElement);
    assert_eq!(7, i.invoke(check, 21).unwrap());
    assert!(i.invoke::<i32, i32>(check, 22).err().unwrap() == RuntimeError::UninitializedElement);
    assert_eq!(8, i.invoke(check, 23).unwrap());
    assert_eq!(8, i.invoke(check, 24).unwrap());
    assert!(i.invoke::<i32, i32>(check, 25).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 26).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 27).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 28).err().unwrap() == RuntimeError::UninitializedElement);
    assert!(i.invoke::<i32, i32>(check, 29).err().unwrap() == RuntimeError::UninitializedElement);
}

#[test_log::test]
fn table_init_6_test() {
    let w = r#"
(module
    (func (export "test")
(elem.drop 0)))
        "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(validate(&wasm_bytes).err().unwrap() == Error::ElementIsNotDefined(0));
}

#[test_log::test]
fn table_init_7_test() {
    let w = r#"
  (module
    (func (export "test")
      (table.init 0 (i32.const 12) (i32.const 1) (i32.const 1))))
        "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(validate(&wasm_bytes).err().unwrap() == Error::TableIsNotDefined(0));
}

#[test_log::test]
fn table_init_8_test() {
    let w = r#"
  (module
    (elem funcref (ref.func 0))
    (func (result i32) (i32.const 0))
    (func (export "test")
      (elem.drop 4)))
        "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(validate(&wasm_bytes).err().unwrap() == Error::ElementIsNotDefined(4));
}

#[test_log::test]
fn table_init_9_test() {
    let w = r#"
  (module
    (elem funcref (ref.func 0))
    (func (result i32) (i32.const 0))
    (func (export "test")
      (table.init 4 (i32.const 12) (i32.const 1) (i32.const 1))))
        "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(validate(&wasm_bytes).err().unwrap() == Error::TableIsNotDefined(0));
}

#[test_log::test]
fn table_init_10_test() {
    let w = r#"
(module
  (table $t0 30 30 funcref)
  (table $t1 28 28 funcref)
  (elem (table $t0) (i32.const 2) func 3 1 4 1)
  (elem funcref
    (ref.func 2) (ref.func 7) (ref.func 1) (ref.func 8))
  (elem (table $t0) (i32.const 12) func 7 5 2 3 6)
  (elem funcref
    (ref.func 5) (ref.func 9) (ref.func 2) (ref.func 7) (ref.func 6))
  (func (result i32) (i32.const 0))
  (func (result i32) (i32.const 1))
  (func (result i32) (i32.const 2))
  (func (result i32) (i32.const 3))
  (func (result i32) (i32.const 4))
  (func (result i32) (i32.const 5))
  (func (result i32) (i32.const 6))
  (func (result i32) (i32.const 7))
  (func (result i32) (i32.const 8))
  (func (result i32) (i32.const 9))
  (func (export "test")
    (elem.drop 2)
    ))
        "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut i = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let test = get_func!(i, "test");

    i.invoke::<(), ()>(test, ()).unwrap();
}

#[test_log::test]
fn table_init_11_test() {
    let w = r#"
(module
  (table $t0 30 30 funcref)
  (table $t1 28 28 funcref)
  (elem (table $t0) (i32.const 2) func 3 1 4 1)
  (elem funcref
    (ref.func 2) (ref.func 7) (ref.func 1) (ref.func 8))
  (elem (table $t0) (i32.const 12) func 7 5 2 3 6)
  (elem funcref
    (ref.func 5) (ref.func 9) (ref.func 2) (ref.func 7) (ref.func 6))
  (func (result i32) (i32.const 0))
  (func (result i32) (i32.const 1))
  (func (result i32) (i32.const 2))
  (func (result i32) (i32.const 3))
  (func (result i32) (i32.const 4))
  (func (result i32) (i32.const 5))
  (func (result i32) (i32.const 6))
  (func (result i32) (i32.const 7))
  (func (result i32) (i32.const 8))
  (func (result i32) (i32.const 9))
  (func (export "test")
    (table.init 2 (i32.const 12) (i32.const 1) (i32.const 1))
    ))
        "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut i = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let test = get_func!(i, "test");

    assert!(i.invoke::<(), ()>(test, ()).err().unwrap() == RuntimeError::TableAccessOutOfBounds);
}

#[test_log::test]
fn table_init_12_test() {
    let w = r#"
(module
  (table $t0 30 30 funcref)
  (table $t1 28 28 funcref)
  (elem (table $t0) (i32.const 2) func 3 1 4 1)
  (elem funcref
    (ref.func 2) (ref.func 7) (ref.func 1) (ref.func 8))
  (elem (table $t0) (i32.const 12) func 7 5 2 3 6)
  (elem funcref
    (ref.func 5) (ref.func 9) (ref.func 2) (ref.func 7) (ref.func 6))
  (func (result i32) (i32.const 0))
  (func (result i32) (i32.const 1))
  (func (result i32) (i32.const 2))
  (func (result i32) (i32.const 3))
  (func (result i32) (i32.const 4))
  (func (result i32) (i32.const 5))
  (func (result i32) (i32.const 6))
  (func (result i32) (i32.const 7))
  (func (result i32) (i32.const 8))
  (func (result i32) (i32.const 9))
  (func (export "test")
    (table.init 1 (i32.const 12) (i32.const 1) (i32.const 1))
    (table.init 1 (i32.const 21) (i32.const 1) (i32.const 1))))
        "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut i = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let test = get_func!(i, "test");

    i.invoke::<(), ()>(test, ()).unwrap();
}

#[test_log::test]
fn table_init_13_test() {
    let w = r#"
(module
  (table $t0 30 30 funcref)
  (table $t1 28 28 funcref)
  (elem (table $t0) (i32.const 2) func 3 1 4 1)
  (elem funcref
    (ref.func 2) (ref.func 7) (ref.func 1) (ref.func 8))
  (elem (table $t0) (i32.const 12) func 7 5 2 3 6)
  (elem funcref
    (ref.func 5) (ref.func 9) (ref.func 2) (ref.func 7) (ref.func 6))
  (func (result i32) (i32.const 0))
  (func (result i32) (i32.const 1))
  (func (result i32) (i32.const 2))
  (func (result i32) (i32.const 3))
  (func (result i32) (i32.const 4))
  (func (result i32) (i32.const 5))
  (func (result i32) (i32.const 6))
  (func (result i32) (i32.const 7))
  (func (result i32) (i32.const 8))
  (func (result i32) (i32.const 9))
  (func (export "test")
    (elem.drop 1)
    (elem.drop 1)))
        "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut i = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let test = get_func!(i, "test");

    i.invoke::<(), ()>(test, ()).unwrap();
}

#[test_log::test]
fn table_init_14_test() {
    let w = r#"
(module
  (table $t0 30 30 funcref)
  (table $t1 28 28 funcref)
  (elem (table $t0) (i32.const 2) func 3 1 4 1)
  (elem funcref
    (ref.func 2) (ref.func 7) (ref.func 1) (ref.func 8))
  (elem (table $t0) (i32.const 12) func 7 5 2 3 6)
  (elem funcref
    (ref.func 5) (ref.func 9) (ref.func 2) (ref.func 7) (ref.func 6))
  (func (result i32) (i32.const 0))
  (func (result i32) (i32.const 1))
  (func (result i32) (i32.const 2))
  (func (result i32) (i32.const 3))
  (func (result i32) (i32.const 4))
  (func (result i32) (i32.const 5))
  (func (result i32) (i32.const 6))
  (func (result i32) (i32.const 7))
  (func (result i32) (i32.const 8))
  (func (result i32) (i32.const 9))
  (func (export "test")
    (elem.drop 1)
    (table.init 1 (i32.const 12) (i32.const 1) (i32.const 1))))
        "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut i = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let test = get_func!(i, "test");

    assert!(i.invoke::<(), ()>(test, ()).err().unwrap() == RuntimeError::TableAccessOutOfBounds);
}

#[test_log::test]
fn table_init_15_test() {
    let w = r#"
(module
  (table $t0 30 30 funcref)
  (table $t1 28 28 funcref)
  (elem (table $t0) (i32.const 2) func 3 1 4 1)
  (elem funcref
    (ref.func 2) (ref.func 7) (ref.func 1) (ref.func 8))
  (elem (table $t0) (i32.const 12) func 7 5 2 3 6)
  (elem funcref
    (ref.func 5) (ref.func 9) (ref.func 2) (ref.func 7) (ref.func 6))
  (func (result i32) (i32.const 0))
  (func (result i32) (i32.const 1))
  (func (result i32) (i32.const 2))
  (func (result i32) (i32.const 3))
  (func (result i32) (i32.const 4))
  (func (result i32) (i32.const 5))
  (func (result i32) (i32.const 6))
  (func (result i32) (i32.const 7))
  (func (result i32) (i32.const 8))
  (func (result i32) (i32.const 9))
  (func (export "test")
    (table.init 1 (i32.const 12) (i32.const 0) (i32.const 5))
    ))
        "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut i = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let test = get_func!(i, "test");

    assert!(i.invoke::<(), ()>(test, ()).err().unwrap() == RuntimeError::TableAccessOutOfBounds);
}

#[test_log::test]
fn table_init_16_test() {
    let w = r#"
(module
  (table $t0 30 30 funcref)
  (table $t1 28 28 funcref)
  (elem (table $t0) (i32.const 2) func 3 1 4 1)
  (elem funcref
    (ref.func 2) (ref.func 7) (ref.func 1) (ref.func 8))
  (elem (table $t0) (i32.const 12) func 7 5 2 3 6)
  (elem funcref
    (ref.func 5) (ref.func 9) (ref.func 2) (ref.func 7) (ref.func 6))
  (func (result i32) (i32.const 0))
  (func (result i32) (i32.const 1))
  (func (result i32) (i32.const 2))
  (func (result i32) (i32.const 3))
  (func (result i32) (i32.const 4))
  (func (result i32) (i32.const 5))
  (func (result i32) (i32.const 6))
  (func (result i32) (i32.const 7))
  (func (result i32) (i32.const 8))
  (func (result i32) (i32.const 9))
  (func (export "test")
    (table.init 1 (i32.const 12) (i32.const 2) (i32.const 3))
    ))
        "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut i = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let test = get_func!(i, "test");

    assert!(i.invoke::<(), ()>(test, ()).err().unwrap() == RuntimeError::TableAccessOutOfBounds);
}

#[test_log::test]
fn table_init_17_test() {
    let w = r#"
(module
  (table $t0 30 30 funcref)
  (table $t1 28 28 funcref)
  (elem (table $t0) (i32.const 2) func 3 1 4 1)
  (elem funcref
    (ref.func 2) (ref.func 7) (ref.func 1) (ref.func 8))
  (elem (table $t0) (i32.const 12) func 7 5 2 3 6)
  (elem funcref
    (ref.func 5) (ref.func 9) (ref.func 2) (ref.func 7) (ref.func 6))
  (func (result i32) (i32.const 0))
  (func (result i32) (i32.const 1))
  (func (result i32) (i32.const 2))
  (func (result i32) (i32.const 3))
  (func (result i32) (i32.const 4))
  (func (result i32) (i32.const 5))
  (func (result i32) (i32.const 6))
  (func (result i32) (i32.const 7))
  (func (result i32) (i32.const 8))
  (func (result i32) (i32.const 9))
  (func (export "test")
    (table.init $t0 1 (i32.const 28) (i32.const 1) (i32.const 3))
    ))
        "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut i = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let test = get_func!(i, "test");

    assert!(i.invoke::<(), ()>(test, ()).err().unwrap() == RuntimeError::TableAccessOutOfBounds);
}

#[test_log::test]
fn table_init_18_test() {
    let w = r#"
(module
  (table $t0 30 30 funcref)
  (table $t1 28 28 funcref)
  (elem (table $t0) (i32.const 2) func 3 1 4 1)
  (elem funcref
    (ref.func 2) (ref.func 7) (ref.func 1) (ref.func 8))
  (elem (table $t0) (i32.const 12) func 7 5 2 3 6)
  (elem funcref
    (ref.func 5) (ref.func 9) (ref.func 2) (ref.func 7) (ref.func 6))
  (func (result i32) (i32.const 0))
  (func (result i32) (i32.const 1))
  (func (result i32) (i32.const 2))
  (func (result i32) (i32.const 3))
  (func (result i32) (i32.const 4))
  (func (result i32) (i32.const 5))
  (func (result i32) (i32.const 6))
  (func (result i32) (i32.const 7))
  (func (result i32) (i32.const 8))
  (func (result i32) (i32.const 9))
  (func (export "test")
    (table.init $t0 1 (i32.const 12) (i32.const 4) (i32.const 0))
    ))
        "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut i = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let test = get_func!(i, "test");

    i.invoke::<(), ()>(test, ()).unwrap();
}

#[test_log::test]
fn table_init_19_test() {
    let w = r#"
(module
  (table $t0 30 30 funcref)
  (table $t1 28 28 funcref)
  (elem (table $t0) (i32.const 2) func 3 1 4 1)
  (elem funcref
    (ref.func 2) (ref.func 7) (ref.func 1) (ref.func 8))
  (elem (table $t0) (i32.const 12) func 7 5 2 3 6)
  (elem funcref
    (ref.func 5) (ref.func 9) (ref.func 2) (ref.func 7) (ref.func 6))
  (func (result i32) (i32.const 0))
  (func (result i32) (i32.const 1))
  (func (result i32) (i32.const 2))
  (func (result i32) (i32.const 3))
  (func (result i32) (i32.const 4))
  (func (result i32) (i32.const 5))
  (func (result i32) (i32.const 6))
  (func (result i32) (i32.const 7))
  (func (result i32) (i32.const 8))
  (func (result i32) (i32.const 9))
  (func (export "test")
    (table.init $t0 1 (i32.const 12) (i32.const 5) (i32.const 0))
    ))
        "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut i = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let test = get_func!(i, "test");

    assert!(i.invoke::<(), ()>(test, ()).err().unwrap() == RuntimeError::TableAccessOutOfBounds);
}

#[test_log::test]
fn table_init_20_test() {
    let w = r#"
(module
  (table $t0 30 30 funcref)
  (table $t1 28 28 funcref)
  (elem (table $t0) (i32.const 2) func 3 1 4 1)
  (elem funcref
    (ref.func 2) (ref.func 7) (ref.func 1) (ref.func 8))
  (elem (table $t0) (i32.const 12) func 7 5 2 3 6)
  (elem funcref
    (ref.func 5) (ref.func 9) (ref.func 2) (ref.func 7) (ref.func 6))
  (func (result i32) (i32.const 0))
  (func (result i32) (i32.const 1))
  (func (result i32) (i32.const 2))
  (func (result i32) (i32.const 3))
  (func (result i32) (i32.const 4))
  (func (result i32) (i32.const 5))
  (func (result i32) (i32.const 6))
  (func (result i32) (i32.const 7))
  (func (result i32) (i32.const 8))
  (func (result i32) (i32.const 9))
  (func (export "test")
    (table.init $t0 1 (i32.const 30) (i32.const 2) (i32.const 0))
    ))
        "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut i = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let test = get_func!(i, "test");

    i.invoke::<(), ()>(test, ()).unwrap();
}

#[test_log::test]
fn table_init_21_test() {
    let w = r#"
(module
  (table $t0 30 30 funcref)
  (table $t1 28 28 funcref)
  (elem (table $t0) (i32.const 2) func 3 1 4 1)
  (elem funcref
    (ref.func 2) (ref.func 7) (ref.func 1) (ref.func 8))
  (elem (table $t0) (i32.const 12) func 7 5 2 3 6)
  (elem funcref
    (ref.func 5) (ref.func 9) (ref.func 2) (ref.func 7) (ref.func 6))
  (func (result i32) (i32.const 0))
  (func (result i32) (i32.const 1))
  (func (result i32) (i32.const 2))
  (func (result i32) (i32.const 3))
  (func (result i32) (i32.const 4))
  (func (result i32) (i32.const 5))
  (func (result i32) (i32.const 6))
  (func (result i32) (i32.const 7))
  (func (result i32) (i32.const 8))
  (func (result i32) (i32.const 9))
  (func (export "test")
    (table.init $t0 1 (i32.const 31) (i32.const 2) (i32.const 0))
    ))
        "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut i = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let test = get_func!(i, "test");

    assert!(i.invoke::<(), ()>(test, ()).err().unwrap() == RuntimeError::TableAccessOutOfBounds);
}

#[test_log::test]
fn table_init_22_test() {
    let w = r#"
(module
  (table $t0 30 30 funcref)
  (table $t1 28 28 funcref)
  (elem (table $t0) (i32.const 2) func 3 1 4 1)
  (elem funcref
    (ref.func 2) (ref.func 7) (ref.func 1) (ref.func 8))
  (elem (table $t0) (i32.const 12) func 7 5 2 3 6)
  (elem funcref
    (ref.func 5) (ref.func 9) (ref.func 2) (ref.func 7) (ref.func 6))
  (func (result i32) (i32.const 0))
  (func (result i32) (i32.const 1))
  (func (result i32) (i32.const 2))
  (func (result i32) (i32.const 3))
  (func (result i32) (i32.const 4))
  (func (result i32) (i32.const 5))
  (func (result i32) (i32.const 6))
  (func (result i32) (i32.const 7))
  (func (result i32) (i32.const 8))
  (func (result i32) (i32.const 9))
  (func (export "test")
    (table.init $t0 1 (i32.const 30) (i32.const 4) (i32.const 0))
    ))
        "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut i = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let test = get_func!(i, "test");

    i.invoke::<(), ()>(test, ()).unwrap();
}

#[test_log::test]
fn table_init_23_test() {
    let w = r#"
(module
  (table $t0 30 30 funcref)
  (table $t1 28 28 funcref)
  (elem (table $t0) (i32.const 2) func 3 1 4 1)
  (elem funcref
    (ref.func 2) (ref.func 7) (ref.func 1) (ref.func 8))
  (elem (table $t0) (i32.const 12) func 7 5 2 3 6)
  (elem funcref
    (ref.func 5) (ref.func 9) (ref.func 2) (ref.func 7) (ref.func 6))
  (func (result i32) (i32.const 0))
  (func (result i32) (i32.const 1))
  (func (result i32) (i32.const 2))
  (func (result i32) (i32.const 3))
  (func (result i32) (i32.const 4))
  (func (result i32) (i32.const 5))
  (func (result i32) (i32.const 6))
  (func (result i32) (i32.const 7))
  (func (result i32) (i32.const 8))
  (func (result i32) (i32.const 9))
  (func (export "test")
    (table.init $t0 1 (i32.const 31) (i32.const 5) (i32.const 0))
    ))
        "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut i = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let test = get_func!(i, "test");

    assert!(i.invoke::<(), ()>(test, ()).err().unwrap() == RuntimeError::TableAccessOutOfBounds);
}

#[test_log::test]
fn table_init_24_test() {
    let w = r#"
(module
  (table $t0 30 30 funcref)
  (table $t1 28 28 funcref)
  (elem (table $t1) (i32.const 2) func 3 1 4 1)
  (elem funcref
    (ref.func 2) (ref.func 7) (ref.func 1) (ref.func 8))
  (elem (table $t1) (i32.const 12) func 7 5 2 3 6)
  (elem funcref
    (ref.func 5) (ref.func 9) (ref.func 2) (ref.func 7) (ref.func 6))
  (func (result i32) (i32.const 0))
  (func (result i32) (i32.const 1))
  (func (result i32) (i32.const 2))
  (func (result i32) (i32.const 3))
  (func (result i32) (i32.const 4))
  (func (result i32) (i32.const 5))
  (func (result i32) (i32.const 6))
  (func (result i32) (i32.const 7))
  (func (result i32) (i32.const 8))
  (func (result i32) (i32.const 9))
  (func (export "test")
    (table.init $t1 1 (i32.const 26) (i32.const 1) (i32.const 3))
    ))
        "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut i = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let test = get_func!(i, "test");

    assert!(i.invoke::<(), ()>(test, ()).err().unwrap() == RuntimeError::TableAccessOutOfBounds);
}

#[test_log::test]
fn table_init_25_test() {
    let w = r#"
(module
  (table $t0 30 30 funcref)
  (table $t1 28 28 funcref)
  (elem (table $t1) (i32.const 2) func 3 1 4 1)
  (elem funcref
    (ref.func 2) (ref.func 7) (ref.func 1) (ref.func 8))
  (elem (table $t1) (i32.const 12) func 7 5 2 3 6)
  (elem funcref
    (ref.func 5) (ref.func 9) (ref.func 2) (ref.func 7) (ref.func 6))
  (func (result i32) (i32.const 0))
  (func (result i32) (i32.const 1))
  (func (result i32) (i32.const 2))
  (func (result i32) (i32.const 3))
  (func (result i32) (i32.const 4))
  (func (result i32) (i32.const 5))
  (func (result i32) (i32.const 6))
  (func (result i32) (i32.const 7))
  (func (result i32) (i32.const 8))
  (func (result i32) (i32.const 9))
  (func (export "test")
    (table.init $t1 1 (i32.const 12) (i32.const 4) (i32.const 0))
    ))
        "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut i = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let test = get_func!(i, "test");

    i.invoke::<(), ()>(test, ()).unwrap();
}

#[test_log::test]
fn table_init_26_test() {
    let w = r#"
(module
  (table $t0 30 30 funcref)
  (table $t1 28 28 funcref)
  (elem (table $t1) (i32.const 2) func 3 1 4 1)
  (elem funcref
    (ref.func 2) (ref.func 7) (ref.func 1) (ref.func 8))
  (elem (table $t1) (i32.const 12) func 7 5 2 3 6)
  (elem funcref
    (ref.func 5) (ref.func 9) (ref.func 2) (ref.func 7) (ref.func 6))
  (func (result i32) (i32.const 0))
  (func (result i32) (i32.const 1))
  (func (result i32) (i32.const 2))
  (func (result i32) (i32.const 3))
  (func (result i32) (i32.const 4))
  (func (result i32) (i32.const 5))
  (func (result i32) (i32.const 6))
  (func (result i32) (i32.const 7))
  (func (result i32) (i32.const 8))
  (func (result i32) (i32.const 9))
  (func (export "test")
    (table.init $t1 1 (i32.const 12) (i32.const 5) (i32.const 0))
    ))
        "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut i = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let test = get_func!(i, "test");

    assert!(i.invoke::<(), ()>(test, ()).err().unwrap() == RuntimeError::TableAccessOutOfBounds);
}

#[test_log::test]
fn table_init_27_test() {
    let w = r#"
(module
  (table $t0 30 30 funcref)
  (table $t1 28 28 funcref)
  (elem (table $t1) (i32.const 2) func 3 1 4 1)
  (elem funcref
    (ref.func 2) (ref.func 7) (ref.func 1) (ref.func 8))
  (elem (table $t1) (i32.const 12) func 7 5 2 3 6)
  (elem funcref
    (ref.func 5) (ref.func 9) (ref.func 2) (ref.func 7) (ref.func 6))
  (func (result i32) (i32.const 0))
  (func (result i32) (i32.const 1))
  (func (result i32) (i32.const 2))
  (func (result i32) (i32.const 3))
  (func (result i32) (i32.const 4))
  (func (result i32) (i32.const 5))
  (func (result i32) (i32.const 6))
  (func (result i32) (i32.const 7))
  (func (result i32) (i32.const 8))
  (func (result i32) (i32.const 9))
  (func (export "test")
    (table.init $t1 1 (i32.const 28) (i32.const 2) (i32.const 0))
    ))
        "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut i = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let test = get_func!(i, "test");

    i.invoke::<(), ()>(test, ()).unwrap();
}

#[test_log::test]
fn table_init_28_test() {
    let w = r#"
(module
  (table $t0 30 30 funcref)
  (table $t1 28 28 funcref)
  (elem (table $t1) (i32.const 2) func 3 1 4 1)
  (elem funcref
    (ref.func 2) (ref.func 7) (ref.func 1) (ref.func 8))
  (elem (table $t1) (i32.const 12) func 7 5 2 3 6)
  (elem funcref
    (ref.func 5) (ref.func 9) (ref.func 2) (ref.func 7) (ref.func 6))
  (func (result i32) (i32.const 0))
  (func (result i32) (i32.const 1))
  (func (result i32) (i32.const 2))
  (func (result i32) (i32.const 3))
  (func (result i32) (i32.const 4))
  (func (result i32) (i32.const 5))
  (func (result i32) (i32.const 6))
  (func (result i32) (i32.const 7))
  (func (result i32) (i32.const 8))
  (func (result i32) (i32.const 9))
  (func (export "test")
    (table.init $t1 1 (i32.const 29) (i32.const 2) (i32.const 0))
    ))
        "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut i = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let test = get_func!(i, "test");

    assert!(i.invoke::<(), ()>(test, ()).err().unwrap() == RuntimeError::TableAccessOutOfBounds);
}

#[test_log::test]
fn table_init_29_test() {
    let w = r#"
(module
  (table $t0 30 30 funcref)
  (table $t1 28 28 funcref)
  (elem (table $t1) (i32.const 2) func 3 1 4 1)
  (elem funcref
    (ref.func 2) (ref.func 7) (ref.func 1) (ref.func 8))
  (elem (table $t1) (i32.const 12) func 7 5 2 3 6)
  (elem funcref
    (ref.func 5) (ref.func 9) (ref.func 2) (ref.func 7) (ref.func 6))
  (func (result i32) (i32.const 0))
  (func (result i32) (i32.const 1))
  (func (result i32) (i32.const 2))
  (func (result i32) (i32.const 3))
  (func (result i32) (i32.const 4))
  (func (result i32) (i32.const 5))
  (func (result i32) (i32.const 6))
  (func (result i32) (i32.const 7))
  (func (result i32) (i32.const 8))
  (func (result i32) (i32.const 9))
  (func (export "test")
    (table.init $t1 1 (i32.const 28) (i32.const 4) (i32.const 0))
    ))
        "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut i = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let test = get_func!(i, "test");

    i.invoke::<(), ()>(test, ()).unwrap();
}

#[test_log::test]
fn table_init_30_test() {
    let w = r#"
(module
  (table $t0 30 30 funcref)
  (table $t1 28 28 funcref)
  (elem (table $t1) (i32.const 2) func 3 1 4 1)
  (elem funcref
    (ref.func 2) (ref.func 7) (ref.func 1) (ref.func 8))
  (elem (table $t1) (i32.const 12) func 7 5 2 3 6)
  (elem funcref
    (ref.func 5) (ref.func 9) (ref.func 2) (ref.func 7) (ref.func 6))
  (func (result i32) (i32.const 0))
  (func (result i32) (i32.const 1))
  (func (result i32) (i32.const 2))
  (func (result i32) (i32.const 3))
  (func (result i32) (i32.const 4))
  (func (result i32) (i32.const 5))
  (func (result i32) (i32.const 6))
  (func (result i32) (i32.const 7))
  (func (result i32) (i32.const 8))
  (func (result i32) (i32.const 9))
  (func (export "test")
    (table.init $t1 1 (i32.const 29) (i32.const 5) (i32.const 0))
    ))
        "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut i = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let test = get_func!(i, "test");

    assert!(i.invoke::<(), ()>(test, ()).err().unwrap() == RuntimeError::TableAccessOutOfBounds);
}

#[test_log::test]
fn table_init_31_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (i32.const 1) (i32.const 1) (f32.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_32_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (i32.const 1) (i32.const 1) (i64.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_33_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (i32.const 1) (i32.const 1) (f64.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_34_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (i32.const 1) (f32.const 1) (i32.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_35_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (i32.const 1) (f32.const 1) (f32.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_36_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (i32.const 1) (f32.const 1) (i64.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_37_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (i32.const 1) (f32.const 1) (f64.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_38_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (i32.const 1) (i64.const 1) (i32.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_39_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (i32.const 1) (i64.const 1) (f32.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_40_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (i32.const 1) (i64.const 1) (i64.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_41_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (i32.const 1) (i64.const 1) (f64.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_42_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (i32.const 1) (f64.const 1) (i32.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_43_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (i32.const 1) (f64.const 1) (f32.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_44_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (i32.const 1) (f64.const 1) (i64.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_45_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (i32.const 1) (f64.const 1) (f64.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_46_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (f32.const 1) (i32.const 1) (i32.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_47_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (f32.const 1) (i32.const 1) (f32.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_48_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (f32.const 1) (i32.const 1) (i64.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_49_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (f32.const 1) (i32.const 1) (f64.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_50_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (f32.const 1) (f32.const 1) (i32.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_51_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (f32.const 1) (f32.const 1) (f32.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_52_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (f32.const 1) (f32.const 1) (i64.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_53_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (f32.const 1) (f32.const 1) (f64.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_54_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (f32.const 1) (i64.const 1) (i32.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_55_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (f32.const 1) (i64.const 1) (f32.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_56_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (f32.const 1) (i64.const 1) (i64.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_57_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (f32.const 1) (i64.const 1) (f64.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_58_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (f32.const 1) (f64.const 1) (i32.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_59_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (f32.const 1) (f64.const 1) (f32.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_60_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (f32.const 1) (f64.const 1) (i64.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_61_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (f32.const 1) (f64.const 1) (f64.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_62_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (i64.const 1) (i32.const 1) (i32.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_63_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (i64.const 1) (i32.const 1) (f32.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_64_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (i64.const 1) (i32.const 1) (i64.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_65_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (i64.const 1) (i32.const 1) (f64.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_66_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (i64.const 1) (f32.const 1) (i32.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_67_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (i64.const 1) (f32.const 1) (f32.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_68_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (i64.const 1) (f32.const 1) (i64.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_69_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (i64.const 1) (f32.const 1) (f64.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_70_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (i64.const 1) (i64.const 1) (i32.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_71_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (i64.const 1) (i64.const 1) (f32.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_72_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (i64.const 1) (i64.const 1) (i64.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_73_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (i64.const 1) (i64.const 1) (f64.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_74_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (i64.const 1) (f64.const 1) (i32.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_75_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (i64.const 1) (f64.const 1) (f32.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_76_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (i64.const 1) (f64.const 1) (i64.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_77_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (i64.const 1) (f64.const 1) (f64.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_78_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (f64.const 1) (i32.const 1) (i32.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_79_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (f64.const 1) (i32.const 1) (f32.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_80_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (f64.const 1) (i32.const 1) (i64.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_81_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (f64.const 1) (i32.const 1) (f64.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_82_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (f64.const 1) (f32.const 1) (i32.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_83_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (f64.const 1) (f32.const 1) (f32.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_84_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (f64.const 1) (f32.const 1) (i64.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_85_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (f64.const 1) (f32.const 1) (f64.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_86_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (f64.const 1) (i64.const 1) (i32.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_87_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (f64.const 1) (i64.const 1) (f32.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_88_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (f64.const 1) (i64.const 1) (i64.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_89_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (f64.const 1) (i64.const 1) (f64.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_90_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (f64.const 1) (f64.const 1) (i32.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_91_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (f64.const 1) (f64.const 1) (f32.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_92_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (f64.const 1) (f64.const 1) (i64.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_93_test() {
    let w = r#"
  (module
    (table 10 funcref)
    (elem funcref (ref.func $f0) (ref.func $f0) (ref.func $f0))
    (func $f0)
    (func (export "test")
      (table.init 0 (f64.const 1) (f64.const 1) (f64.const 1))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    assert!(matches!(
        validate(&wasm_bytes).err().unwrap(),
        Error::InvalidValidationStackValType(_)
    ));
}

#[test_log::test]
fn table_init_94_test() {
    let w = r#"
(module
  (type (func (result i32)))
  (table 32 64 funcref)
  (elem funcref
    (ref.func $f0) (ref.func $f1) (ref.func $f2) (ref.func $f3)
    (ref.func $f4) (ref.func $f5) (ref.func $f6) (ref.func $f7)
    (ref.func $f8) (ref.func $f9) (ref.func $f10) (ref.func $f11)
    (ref.func $f12) (ref.func $f13) (ref.func $f14) (ref.func $f15))
  (func $f0 (export "f0") (result i32) (i32.const 0))
  (func $f1 (export "f1") (result i32) (i32.const 1))
  (func $f2 (export "f2") (result i32) (i32.const 2))
  (func $f3 (export "f3") (result i32) (i32.const 3))
  (func $f4 (export "f4") (result i32) (i32.const 4))
  (func $f5 (export "f5") (result i32) (i32.const 5))
  (func $f6 (export "f6") (result i32) (i32.const 6))
  (func $f7 (export "f7") (result i32) (i32.const 7))
  (func $f8 (export "f8") (result i32) (i32.const 8))
  (func $f9 (export "f9") (result i32) (i32.const 9))
  (func $f10 (export "f10") (result i32) (i32.const 10))
  (func $f11 (export "f11") (result i32) (i32.const 11))
  (func $f12 (export "f12") (result i32) (i32.const 12))
  (func $f13 (export "f13") (result i32) (i32.const 13))
  (func $f14 (export "f14") (result i32) (i32.const 14))
  (func $f15 (export "f15") (result i32) (i32.const 15))
  (func (export "test") (param $n i32) (result i32)
    (call_indirect (type 0) (local.get $n)))
  (func (export "run") (param $offs i32) (param $len i32)
    (table.init 0 (local.get $offs) (i32.const 0) (local.get $len))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut inst = RuntimeInstance::new(&validation_info).unwrap();

    let run = get_func!(inst, "run");
    let test = get_func!(inst, "test");
    assert!(
        inst.invoke::<(i32, i32), ()>(run, (24, 16)).err().unwrap()
            == RuntimeError::TableAccessOutOfBounds
    );
    for i in 0..32 {
        assert!(
            inst.invoke::<i32, i32>(test, i).err().unwrap() == RuntimeError::UninitializedElement
        );
    }
}

#[test_log::test]
fn table_init_95_test() {
    let w = r#"
(module
  (type (func (result i32)))
  (table 32 64 funcref)
  (elem funcref
    (ref.func $f0) (ref.func $f1) (ref.func $f2) (ref.func $f3)
    (ref.func $f4) (ref.func $f5) (ref.func $f6) (ref.func $f7)
    (ref.func $f8) (ref.func $f9) (ref.func $f10) (ref.func $f11)
    (ref.func $f12) (ref.func $f13) (ref.func $f14) (ref.func $f15))
  (func $f0 (export "f0") (result i32) (i32.const 0))
  (func $f1 (export "f1") (result i32) (i32.const 1))
  (func $f2 (export "f2") (result i32) (i32.const 2))
  (func $f3 (export "f3") (result i32) (i32.const 3))
  (func $f4 (export "f4") (result i32) (i32.const 4))
  (func $f5 (export "f5") (result i32) (i32.const 5))
  (func $f6 (export "f6") (result i32) (i32.const 6))
  (func $f7 (export "f7") (result i32) (i32.const 7))
  (func $f8 (export "f8") (result i32) (i32.const 8))
  (func $f9 (export "f9") (result i32) (i32.const 9))
  (func $f10 (export "f10") (result i32) (i32.const 10))
  (func $f11 (export "f11") (result i32) (i32.const 11))
  (func $f12 (export "f12") (result i32) (i32.const 12))
  (func $f13 (export "f13") (result i32) (i32.const 13))
  (func $f14 (export "f14") (result i32) (i32.const 14))
  (func $f15 (export "f15") (result i32) (i32.const 15))
  (func (export "test") (param $n i32) (result i32)
    (call_indirect (type 0) (local.get $n)))
  (func (export "run") (param $offs i32) (param $len i32)
    (table.init 0 (local.get $offs) (i32.const 0) (local.get $len))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut inst = RuntimeInstance::new(&validation_info).unwrap();

    let run = get_func!(inst, "run");
    let test = get_func!(inst, "test");
    assert!(
        inst.invoke::<(i32, i32), ()>(run, (25, 16)).err().unwrap()
            == RuntimeError::TableAccessOutOfBounds
    );
    for i in 0..32 {
        assert!(
            inst.invoke::<i32, i32>(test, i).err().unwrap() == RuntimeError::UninitializedElement
        );
    }
}

#[test_log::test]
fn table_init_96_test() {
    let w = r#"
(module
  (type (func (result i32)))
  (table 160 320 funcref)
  (elem funcref
    (ref.func $f0) (ref.func $f1) (ref.func $f2) (ref.func $f3)
    (ref.func $f4) (ref.func $f5) (ref.func $f6) (ref.func $f7)
    (ref.func $f8) (ref.func $f9) (ref.func $f10) (ref.func $f11)
    (ref.func $f12) (ref.func $f13) (ref.func $f14) (ref.func $f15))
  (func $f0 (export "f0") (result i32) (i32.const 0))
  (func $f1 (export "f1") (result i32) (i32.const 1))
  (func $f2 (export "f2") (result i32) (i32.const 2))
  (func $f3 (export "f3") (result i32) (i32.const 3))
  (func $f4 (export "f4") (result i32) (i32.const 4))
  (func $f5 (export "f5") (result i32) (i32.const 5))
  (func $f6 (export "f6") (result i32) (i32.const 6))
  (func $f7 (export "f7") (result i32) (i32.const 7))
  (func $f8 (export "f8") (result i32) (i32.const 8))
  (func $f9 (export "f9") (result i32) (i32.const 9))
  (func $f10 (export "f10") (result i32) (i32.const 10))
  (func $f11 (export "f11") (result i32) (i32.const 11))
  (func $f12 (export "f12") (result i32) (i32.const 12))
  (func $f13 (export "f13") (result i32) (i32.const 13))
  (func $f14 (export "f14") (result i32) (i32.const 14))
  (func $f15 (export "f15") (result i32) (i32.const 15))
  (func (export "test") (param $n i32) (result i32)
    (call_indirect (type 0) (local.get $n)))
  (func (export "run") (param $offs i32) (param $len i32)
    (table.init 0 (local.get $offs) (i32.const 0) (local.get $len))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut inst = RuntimeInstance::new(&validation_info).unwrap();

    let run = get_func!(inst, "run");
    let test = get_func!(inst, "test");
    assert!(
        inst.invoke::<(i32, i32), ()>(run, (96, 32)).err().unwrap()
            == RuntimeError::TableAccessOutOfBounds
    );
    for i in 0..160 {
        assert!(
            inst.invoke::<i32, i32>(test, i).err().unwrap() == RuntimeError::UninitializedElement
        );
    }
}

#[test_log::test]
fn table_init_97_test() {
    let w = r#"
(module
  (type (func (result i32)))
  (table 160 320 funcref)
  (elem funcref
    (ref.func $f0) (ref.func $f1) (ref.func $f2) (ref.func $f3)
    (ref.func $f4) (ref.func $f5) (ref.func $f6) (ref.func $f7)
    (ref.func $f8) (ref.func $f9) (ref.func $f10) (ref.func $f11)
    (ref.func $f12) (ref.func $f13) (ref.func $f14) (ref.func $f15))
  (func $f0 (export "f0") (result i32) (i32.const 0))
  (func $f1 (export "f1") (result i32) (i32.const 1))
  (func $f2 (export "f2") (result i32) (i32.const 2))
  (func $f3 (export "f3") (result i32) (i32.const 3))
  (func $f4 (export "f4") (result i32) (i32.const 4))
  (func $f5 (export "f5") (result i32) (i32.const 5))
  (func $f6 (export "f6") (result i32) (i32.const 6))
  (func $f7 (export "f7") (result i32) (i32.const 7))
  (func $f8 (export "f8") (result i32) (i32.const 8))
  (func $f9 (export "f9") (result i32) (i32.const 9))
  (func $f10 (export "f10") (result i32) (i32.const 10))
  (func $f11 (export "f11") (result i32) (i32.const 11))
  (func $f12 (export "f12") (result i32) (i32.const 12))
  (func $f13 (export "f13") (result i32) (i32.const 13))
  (func $f14 (export "f14") (result i32) (i32.const 14))
  (func $f15 (export "f15") (result i32) (i32.const 15))
  (func (export "test") (param $n i32) (result i32)
    (call_indirect (type 0) (local.get $n)))
  (func (export "run") (param $offs i32) (param $len i32)
    (table.init 0 (local.get $offs) (i32.const 0) (local.get $len))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut inst = RuntimeInstance::new(&validation_info).unwrap();

    let run = get_func!(inst, "run");
    let test = get_func!(inst, "test");
    assert!(
        inst.invoke::<(i32, i32), ()>(run, (97, 31)).err().unwrap()
            == RuntimeError::TableAccessOutOfBounds
    );
    for i in 0..160 {
        assert!(
            inst.invoke::<i32, i32>(test, i).err().unwrap() == RuntimeError::UninitializedElement
        );
    }
}

#[test_log::test]
fn table_init_98_test() {
    let w = r#"
(module
  (type (func (result i32)))
  (table 64 64 funcref)
  (elem funcref
    (ref.func $f0) (ref.func $f1) (ref.func $f2) (ref.func $f3)
    (ref.func $f4) (ref.func $f5) (ref.func $f6) (ref.func $f7)
    (ref.func $f8) (ref.func $f9) (ref.func $f10) (ref.func $f11)
    (ref.func $f12) (ref.func $f13) (ref.func $f14) (ref.func $f15))
  (func $f0 (export "f0") (result i32) (i32.const 0))
  (func $f1 (export "f1") (result i32) (i32.const 1))
  (func $f2 (export "f2") (result i32) (i32.const 2))
  (func $f3 (export "f3") (result i32) (i32.const 3))
  (func $f4 (export "f4") (result i32) (i32.const 4))
  (func $f5 (export "f5") (result i32) (i32.const 5))
  (func $f6 (export "f6") (result i32) (i32.const 6))
  (func $f7 (export "f7") (result i32) (i32.const 7))
  (func $f8 (export "f8") (result i32) (i32.const 8))
  (func $f9 (export "f9") (result i32) (i32.const 9))
  (func $f10 (export "f10") (result i32) (i32.const 10))
  (func $f11 (export "f11") (result i32) (i32.const 11))
  (func $f12 (export "f12") (result i32) (i32.const 12))
  (func $f13 (export "f13") (result i32) (i32.const 13))
  (func $f14 (export "f14") (result i32) (i32.const 14))
  (func $f15 (export "f15") (result i32) (i32.const 15))
  (func (export "test") (param $n i32) (result i32)
    (call_indirect (type 0) (local.get $n)))
  (func (export "run") (param $offs i32) (param $len i32)
    (table.init 0 (local.get $offs) (i32.const 0) (local.get $len))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut inst = RuntimeInstance::new(&validation_info).unwrap();

    let run = get_func!(inst, "run");
    let test = get_func!(inst, "test");
    assert!(
        inst.invoke::<(i32, u32), ()>(run, (48, 4294967280_u32))
            .err()
            .unwrap()
            == RuntimeError::TableAccessOutOfBounds
    );
    for i in 0..64 {
        assert!(
            inst.invoke::<i32, i32>(test, i).err().unwrap() == RuntimeError::UninitializedElement
        );
    }
}

#[test_log::test]
fn table_init_99_test() {
    let w = r#"
(module
  (type (func (result i32)))
  (table 16 16 funcref)
  (elem funcref
    (ref.func $f0) (ref.func $f1) (ref.func $f2) (ref.func $f3)
    (ref.func $f4) (ref.func $f5) (ref.func $f6) (ref.func $f7)
    (ref.func $f8) (ref.func $f9) (ref.func $f10) (ref.func $f11)
    (ref.func $f12) (ref.func $f13) (ref.func $f14) (ref.func $f15))
  (func $f0 (export "f0") (result i32) (i32.const 0))
  (func $f1 (export "f1") (result i32) (i32.const 1))
  (func $f2 (export "f2") (result i32) (i32.const 2))
  (func $f3 (export "f3") (result i32) (i32.const 3))
  (func $f4 (export "f4") (result i32) (i32.const 4))
  (func $f5 (export "f5") (result i32) (i32.const 5))
  (func $f6 (export "f6") (result i32) (i32.const 6))
  (func $f7 (export "f7") (result i32) (i32.const 7))
  (func $f8 (export "f8") (result i32) (i32.const 8))
  (func $f9 (export "f9") (result i32) (i32.const 9))
  (func $f10 (export "f10") (result i32) (i32.const 10))
  (func $f11 (export "f11") (result i32) (i32.const 11))
  (func $f12 (export "f12") (result i32) (i32.const 12))
  (func $f13 (export "f13") (result i32) (i32.const 13))
  (func $f14 (export "f14") (result i32) (i32.const 14))
  (func $f15 (export "f15") (result i32) (i32.const 15))
  (func (export "test") (param $n i32) (result i32)
    (call_indirect (type 0) (local.get $n)))
  (func (export "run") (param $offs i32) (param $len i32)
    (table.init 0 (local.get $offs) (i32.const 8) (local.get $len))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut inst = RuntimeInstance::new(&validation_info).unwrap();

    let run = get_func!(inst, "run");
    let test = get_func!(inst, "test");
    assert!(
        inst.invoke::<(i32, i32), ()>(run, (0, 4294967292_u32 as i32))
            .err()
            .unwrap()
            == RuntimeError::TableAccessOutOfBounds
    );
    for i in 0..16 {
        assert!(
            inst.invoke::<i32, i32>(test, i).err().unwrap() == RuntimeError::UninitializedElement
        );
    }
}

#[test_log::test]
fn table_init_100_test() {
    let w = r#"
(module
  (table 1 funcref)
  ;; 65 elem segments. 64 is the smallest positive number that is encoded
  ;; differently as a signed LEB.
  (elem funcref) (elem funcref) (elem funcref) (elem funcref)
  (elem funcref) (elem funcref) (elem funcref) (elem funcref)
  (elem funcref) (elem funcref) (elem funcref) (elem funcref)
  (elem funcref) (elem funcref) (elem funcref) (elem funcref)
  (elem funcref) (elem funcref) (elem funcref) (elem funcref)
  (elem funcref) (elem funcref) (elem funcref) (elem funcref)
  (elem funcref) (elem funcref) (elem funcref) (elem funcref)
  (elem funcref) (elem funcref) (elem funcref) (elem funcref)
  (elem funcref) (elem funcref) (elem funcref) (elem funcref)
  (elem funcref) (elem funcref) (elem funcref) (elem funcref)
  (elem funcref) (elem funcref) (elem funcref) (elem funcref)
  (elem funcref) (elem funcref) (elem funcref) (elem funcref)
  (elem funcref) (elem funcref) (elem funcref) (elem funcref)
  (elem funcref) (elem funcref) (elem funcref) (elem funcref)
  (elem funcref) (elem funcref) (elem funcref) (elem funcref)
  (elem funcref) (elem funcref) (elem funcref) (elem funcref)
  (elem funcref)
  (func (table.init 64 (i32.const 0) (i32.const 0) (i32.const 0))))
          "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    RuntimeInstance::new(&validation_info).unwrap();
}
