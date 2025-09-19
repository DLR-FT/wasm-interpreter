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

use wasm::{validate, value::FuncAddr, RuntimeError, RuntimeInstance, TrapError, DEFAULT_MODULE};

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
fn table_funcref_test() {
    let w = r#"
(module
  (table $t2 2 funcref)
  (table $t3 3 funcref)
  (elem (table $t3) (i32.const 1) func $dummy)
  (func $dummy)
  (func (export "init") (param $r funcref)
    (table.set $t2 (i32.const 1) (local.get $r))
    (table.set $t3 (i32.const 2) (table.get $t3 (i32.const 1)))
  )
  (func (export "get-funcref") (param $i i32) (result funcref)
    (table.get (local.get $i))
  )
  (func $f3 (export "get-funcref-2") (param $i i32) (result funcref)
    (table.get $t3 (local.get $i))
  )
  (func (export "is_null-funcref") (param $i i32) (result i32)
    (ref.is_null (call $f3 (local.get $i)))
  )
)
    "#;
    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut i = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");

    let init = get_func!(i, "init");
    let get_funcref = get_func!(i, "get-funcref");
    let get_funcref_2 = get_func!(i, "get-funcref-2");
    let is_null_funcref = get_func!(i, "is_null-funcref");

    i.invoke_typed::<Option<FuncAddr>, ()>(init, Some(FuncAddr(1)))
        .unwrap();

    assert_result!(i, get_funcref, 0, None);
    assert_result!(i, get_funcref, 1, Some(FuncAddr(1)));
    assert_result!(i, get_funcref_2, 0, None);
    assert_result!(i, is_null_funcref, 1, 0);
    assert_result!(i, is_null_funcref, 2, 0);

    assert_error!(
        i,
        get_funcref,
        2,
        Result<Option<FuncAddr>, RuntimeError>,
        i32,
        Option<FuncAddr>,
        RuntimeError::Trap(TrapError::TableOrElementAccessOutOfBounds)
    );
    assert_error!(
        i,
        get_funcref_2,
        3,
        Result<Option<FuncAddr>, RuntimeError>,
        i32,
        Option<FuncAddr>,
        RuntimeError::Trap(TrapError::TableOrElementAccessOutOfBounds)
    );
    assert_error!(
        i,
        get_funcref,
        -1,
        Result<Option<FuncAddr>, RuntimeError>,
        i32,
        Option<FuncAddr>,
        RuntimeError::Trap(TrapError::TableOrElementAccessOutOfBounds)
    );
    assert_error!(
        i,
        get_funcref_2,
        -1,
        Result<Option<FuncAddr>, RuntimeError>,
        i32,
        Option<FuncAddr>,
        RuntimeError::Trap(TrapError::TableOrElementAccessOutOfBounds)
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
        assert!(validate(&wasm_bytes).is_err());
    }
}
