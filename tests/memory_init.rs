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
use wasm::{validate, RuntimeError, RuntimeInstance, TrapError};
use wasm::{ValidationError as GeneralError, DEFAULT_MODULE};

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

macro_rules! assert_error {
    ($instance:expr, $func:expr, $arg:expr, $ret_type:ty, $invoke_param_type:ty, $invoke_return_type:ty, $err_type:expr) => {
        let val: $ret_type =
            $instance.invoke_typed::<$invoke_param_type, $invoke_return_type>($func, $arg);
        assert!(val.is_err());
        assert!(val.unwrap_err() == $err_type);
    };
}

#[test_log::test]
fn memory_init_test_1() {
    let w = r#"
(module
  (memory (export "memory0") 1 1)
  (data (i32.const 2) "\03\01\04\01")
  (data "\02\07\01\08")
  (data (i32.const 12) "\07\05\02\03\06")
  (data "\05\09\02\07\06")
  (func (export "test")
    (nop))
  (func (export "load8_u") (param i32) (result i32)
    (i32.load8_u (local.get 0))))
  "#;
    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut i = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");
    let test = get_func!(i, "test");
    i.invoke_typed::<(), ()>(test, ()).unwrap();

    let load8_u = get_func!(i, "load8_u");

    let offsets = Vec::from([
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29,
    ]);
    let results = Vec::from([
        0, 0, 3, 1, 4, 1, 0, 0, 0, 0, 0, 0, 7, 5, 2, 3, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ]);
    for j in 0..offsets.len() {
        assert_result!(i, load8_u, offsets[j], results[j]);
    }
}

#[test_log::test]
fn memory_init_test_2() {
    let w = r#"
(module
  (memory (export "memory0") 1 1)
  (data (i32.const 2) "\03\01\04\01")
  (data "\02\07\01\08")
  (data (i32.const 12) "\07\05\02\03\06")
  (data "\05\09\02\07\06")
  (func (export "test")
    (memory.init 1 (i32.const 7) (i32.const 0) (i32.const 4)))
  (func (export "load8_u") (param i32) (result i32)
    (i32.load8_u (local.get 0))))
  "#;
    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut i = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");
    let test = get_func!(i, "test");
    i.invoke_typed::<(), ()>(test, ()).unwrap();

    let load8_u = get_func!(i, "load8_u");

    let offsets = Vec::from([
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29,
    ]);
    let results = Vec::from([
        0, 0, 3, 1, 4, 1, 0, 2, 7, 1, 8, 0, 7, 5, 2, 3, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ]);
    for j in 0..offsets.len() {
        assert_result!(i, load8_u, offsets[j], results[j]);
    }
}

#[test_log::test]
fn memory_init_test_3() {
    let w = r#"
(module
  (memory (export "memory0") 1 1)
  (data (i32.const 2) "\03\01\04\01")
  (data "\02\07\01\08")
  (data (i32.const 12) "\07\05\02\03\06")
  (data "\05\09\02\07\06")
  (func (export "test")
    (memory.init 3 (i32.const 15) (i32.const 1) (i32.const 3)))
  (func (export "load8_u") (param i32) (result i32)
    (i32.load8_u (local.get 0))))
  "#;
    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut i = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");
    let test = get_func!(i, "test");
    i.invoke_typed::<(), ()>(test, ()).unwrap();

    let load8_u = get_func!(i, "load8_u");

    let offsets = Vec::from([
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29,
    ]);
    let results = Vec::from([
        0, 0, 3, 1, 4, 1, 0, 0, 0, 0, 0, 0, 7, 5, 2, 9, 2, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ]);
    for j in 0..offsets.len() {
        assert_result!(i, load8_u, offsets[j], results[j]);
    }
}

#[test_log::test]
fn memory_init_test_4() {
    let w = r#"
(module
  (memory (export "memory0") 1 1)
  (data (i32.const 2) "\03\01\04\01")
  (data "\02\07\01\08")
  (data (i32.const 12) "\07\05\02\03\06")
  (data "\05\09\02\07\06")
  (func (export "test")
    (memory.init 1 (i32.const 7) (i32.const 0) (i32.const 4))
    (data.drop 1)
    (memory.init 3 (i32.const 15) (i32.const 1) (i32.const 3))
    (data.drop 3)
    (memory.copy (i32.const 20) (i32.const 15) (i32.const 5))
    (memory.copy (i32.const 21) (i32.const 29) (i32.const 1))
    (memory.copy (i32.const 24) (i32.const 10) (i32.const 1))
    (memory.copy (i32.const 13) (i32.const 11) (i32.const 4))
    (memory.copy (i32.const 19) (i32.const 20) (i32.const 5)))
  (func (export "load8_u") (param i32) (result i32)
    (i32.load8_u (local.get 0))))
  "#;
    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut i = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");
    let test = get_func!(i, "test");
    i.invoke_typed::<(), ()>(test, ()).unwrap();

    let load8_u = get_func!(i, "load8_u");

    let offsets = Vec::from([
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29,
    ]);
    let results = Vec::from([
        0, 0, 3, 1, 4, 1, 0, 2, 7, 1, 8, 0, 7, 0, 7, 5, 2, 7, 0, 9, 0, 7, 0, 8, 8, 0, 0, 0, 0, 0,
    ]);
    for j in 0..offsets.len() {
        assert_result!(i, load8_u, offsets[j], results[j]);
    }
}

#[test_log::test]
fn memory_init_test_5() {
    let w = r#"
   (module
     (func (export "test")
       (data.drop 0)))
  "#;
    let wasm_bytes = wat::parse_str(w).unwrap();
    let res = validate(&wasm_bytes);
    assert!(res.err().unwrap() == GeneralError::DataSegmentNotFound(0));
}

#[test_log::test]
fn memory_init_test_6() {
    let w = r#"
  (module
    (memory 1)
    (data "\37")
    (func (export "test")
      (data.drop 4)))
  "#;
    let wasm_bytes = wat::parse_str(w).unwrap();

    let res = validate(&wasm_bytes);
    assert!(res.err().unwrap() == GeneralError::DataSegmentNotFound(4));
}

#[test_log::test]
fn memory_init_test_7() {
    let w = r#"
(module
  (memory 1)
    (data "\37")
  (func (export "test")
    (data.drop 0)
    (data.drop 0)))
  "#;
    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut i = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");
    let test = get_func!(i, "test");
    i.invoke_typed::<(), ()>(test, ()).unwrap();
}

#[test_log::test]
fn memory_init_test_8() {
    let w = r#"
(module
  (memory 1)
    (data "\37")
  (func (export "test")
    (data.drop 0)
    (memory.init 0 (i32.const 1234) (i32.const 1) (i32.const 1))))
  "#;
    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut i = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");
    assert_error!(i, get_func!(i, "test"), (), Result<(), RuntimeError>, (), (), RuntimeError::Trap(TrapError::MemoryOrDataAccessOutOfBounds));
}

#[test_log::test]
fn memory_init_test_9() {
    let w = r#"
(module
   (memory 1)
   (data (i32.const 0) "\37")
   (func (export "test")
     (memory.init 0 (i32.const 1234) (i32.const 1) (i32.const 1))))
  "#;
    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut i = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");
    assert_error!(i, get_func!(i, "test"), (), Result<(), RuntimeError>, (), (), RuntimeError::Trap(TrapError::MemoryOrDataAccessOutOfBounds));
}

#[test_log::test]
fn memory_init_test_10() {
    let w = r#"
  (module
    (func (export "test")
      (memory.init 1 (i32.const 1234) (i32.const 1) (i32.const 1))))
  "#;
    let wasm_bytes = wat::parse_str(w).unwrap();

    let res = validate(&wasm_bytes);
    assert!(res.err().unwrap() == GeneralError::MemoryIsNotDefined(0));
}

#[test_log::test]
fn memory_init_test_11() {
    let w = r#"
  (module
    (memory 1)
    (data "\37")
    (func (export "test")
      (memory.init 1 (i32.const 1234) (i32.const 1) (i32.const 1))))
  "#;
    let wasm_bytes = wat::parse_str(w).unwrap();

    let res = validate(&wasm_bytes);
    assert!(res.err().unwrap() == GeneralError::DataSegmentNotFound(1));
}

#[test_log::test]
fn memory_init_test_12() {
    let w = r#"
(module
  (memory 1)
    (data "\37")
  (func (export "test")
    (memory.init 0 (i32.const 1) (i32.const 0) (i32.const 1))
    (memory.init 0 (i32.const 1) (i32.const 0) (i32.const 1))))
  "#;
    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut i = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");
    let test = get_func!(i, "test");
    i.invoke_typed::<(), ()>(test, ()).unwrap();
}

#[test_log::test]
fn memory_init_test_13() {
    let w = r#"
(module
    (memory 1)
      (data "\37")
    (func (export "test")
      (memory.init 0 (i32.const 1234) (i32.const 0) (i32.const 5))))
  "#;
    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut i = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");
    assert_error!(i, get_func!(i, "test"), (), Result<(), RuntimeError>, (), (), RuntimeError::Trap(TrapError::MemoryOrDataAccessOutOfBounds));
}
