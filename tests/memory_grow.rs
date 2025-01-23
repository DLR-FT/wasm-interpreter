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
use wasm::{validate, RuntimeError, RuntimeInstance};

macro_rules! get_func {
    ($instance:ident, $func_name:expr) => {
        &$instance.get_function_by_name("", $func_name).unwrap()
    };
}

macro_rules! assert_result {
    ($instance:expr, $func_name:expr, $arg:expr, $result:expr) => {
        assert_eq!($result, $instance.invoke($func_name, $arg).unwrap());
    };
}

macro_rules! assert_error {
    ($instance:expr, $func_name:expr, $arg:expr, $ret_type:ty, $invoke_param_type:ty, $invoke_return_type:ty, $err_type:expr) => {
        let val: $ret_type =
            $instance.invoke::<$invoke_param_type, $invoke_return_type>($func_name, $arg);
        assert!(val.is_err());
        assert!(val.unwrap_err() == $err_type);
    };
}

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
    let mut i = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    // let x = i.invoke(function_ref, params)
    assert_result!(i, get_func!(i, "size"), (), 0);
    assert_error!(i, get_func!(i, "store_at_zero"), (), Result<(), RuntimeError>, (), (), RuntimeError::MemoryAccessOutOfBounds);
    assert_error!(i, get_func!(i, "load_at_zero"), (), Result<i32, RuntimeError>, (), i32, RuntimeError::MemoryAccessOutOfBounds);
    assert_error!(i, get_func!(i, "store_at_page_size"), (), Result<(), RuntimeError>, (), (), RuntimeError::MemoryAccessOutOfBounds);
    assert_error!(i, get_func!(i, "load_at_page_size"), (), Result<i32, RuntimeError>, (), i32, RuntimeError::MemoryAccessOutOfBounds);
    assert_result!(i, get_func!(i, "grow"), 1, 0);
    assert_result!(i, get_func!(i, "size"), (), 1);
    assert_result!(i, get_func!(i, "load_at_zero"), (), 0);
    assert_result!(i, get_func!(i, "store_at_zero"), (), ());
    assert_result!(i, get_func!(i, "load_at_zero"), (), 2);
    assert_error!(i, get_func!(i, "store_at_page_size"), (), Result<(), RuntimeError>, (), (), RuntimeError::MemoryAccessOutOfBounds);
    assert_error!(i, get_func!(i, "load_at_page_size"), (), Result<i32, RuntimeError>, (), i32, RuntimeError::MemoryAccessOutOfBounds);
    assert_result!(i, get_func!(i, "grow"), 4, 1);
    assert_result!(i, get_func!(i, "size"), (), 5);
    assert_result!(i, get_func!(i, "load_at_zero"), (), 2);
    assert_result!(i, get_func!(i, "store_at_zero"), (), ());
    assert_result!(i, get_func!(i, "load_at_zero"), (), 2);
    assert_result!(i, get_func!(i, "load_at_page_size"), (), 0);
    assert_result!(i, get_func!(i, "store_at_page_size"), (), ());
    assert_result!(i, get_func!(i, "load_at_page_size"), (), 3);
}

#[test_log::test]
fn memory_grow_test_2() {
    let w = r#"
    (module
        (memory 0)
        (func (export "grow") (param i32) (result i32) (memory.grow (local.get 0)))
    )
  "#;
    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut i = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_result!(i, get_func!(i, "grow"), 0, 0);
    assert_result!(i, get_func!(i, "grow"), 1, 0);
    assert_result!(i, get_func!(i, "grow"), 0, 1);
    assert_result!(i, get_func!(i, "grow"), 2, 1);
    assert_result!(i, get_func!(i, "grow"), 800, 3);
    assert_result!(i, get_func!(i, "grow"), 0x10000, -1);
    assert_result!(i, get_func!(i, "grow"), 64736, -1);
    assert_result!(i, get_func!(i, "grow"), 1, 803);
}

#[test_log::test]
fn memory_grow_test_3() {
    let w = r#"
    (module
        (memory 0 10)
        (func (export "grow") (param i32) (result i32) (memory.grow (local.get 0)))
    )
  "#;
    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut i = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    assert_result!(i, get_func!(i, "grow"), 0, 0);
    assert_result!(i, get_func!(i, "grow"), 1, 0);
    assert_result!(i, get_func!(i, "grow"), 1, 1);
    assert_result!(i, get_func!(i, "grow"), 2, 2);
    assert_result!(i, get_func!(i, "grow"), 6, 4);
    assert_result!(i, get_func!(i, "grow"), 0, 10);
    assert_result!(i, get_func!(i, "grow"), 1, -1);
    assert_result!(i, get_func!(i, "grow"), 0x10000, -1);
}
