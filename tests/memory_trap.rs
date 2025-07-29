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

macro_rules! assert_error {
    ($instance:expr, $func:expr, $arg:expr, $ret_type:ty, $invoke_param_type:ty, $invoke_return_type:ty, $err_type:expr) => {
        let val: $ret_type =
            $instance.invoke_typed::<$invoke_param_type, $invoke_return_type>($func, $arg);
        assert!(val.is_err());
        assert!(val.unwrap_err() == $err_type);
    };
}

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
    let mut i = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let store = get_func!(i, "store");
    let load = get_func!(i, "load");

    assert_result!(i, store, (-4, 42), ());
    assert_result!(i, load, -4, 42);
    assert_error!(i, store, (-3, 0x12345678), Result<(), RuntimeError>, (i32, i32), (), RuntimeError::MemoryAccessOutOfBounds);
    assert_error!(i, load, -3, Result<i32, RuntimeError>, i32, i32, RuntimeError::MemoryAccessOutOfBounds);
    assert_error!(i, store, (-2, 13), Result<(), RuntimeError>, (i32, i32), (), RuntimeError::MemoryAccessOutOfBounds);
    assert_error!(i, load, -2, Result<i32, RuntimeError>, i32, i32, RuntimeError::MemoryAccessOutOfBounds);
    assert_error!(i, store, (-1, 13), Result<(), RuntimeError>, (i32, i32), (), RuntimeError::MemoryAccessOutOfBounds);
    assert_error!(i, load, -1, Result<i32, RuntimeError>, i32, i32, RuntimeError::MemoryAccessOutOfBounds);
    assert_error!(i, store, (0, 13), Result<(), RuntimeError>, (i32, i32), (), RuntimeError::MemoryAccessOutOfBounds);
    assert_error!(i, load, 0, Result<i32, RuntimeError>, i32, i32, RuntimeError::MemoryAccessOutOfBounds);
    assert_error!(i, store, (0x80000000_u32 as i32, 13), Result<(), RuntimeError>, (i32, i32), (), RuntimeError::MemoryAccessOutOfBounds);
    assert_error!(i, load, 0x80000000_u32 as i32, Result<i32, RuntimeError>, i32, i32, RuntimeError::MemoryAccessOutOfBounds);
    assert_result!(i, get_func!(i, "memory.grow"), 0x10001, -1);
}
