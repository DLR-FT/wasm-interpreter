use wasm::interop::RefFunc;
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
use wasm::value::FuncAddr;
use wasm::{validate, RuntimeError, RuntimeInstance, TrapError, DEFAULT_MODULE};

macro_rules! get_func {
    ($instance:ident, $func_name:expr) => {
        &$instance
            .get_function_by_name(DEFAULT_MODULE, $func_name)
            .unwrap()
    };
}

#[test_log::test]
fn table_fill_test() {
    let w = r#"
    (module
      (table $t 10 funcref)
    
      (func (export "fill") (param $i i32) (param $r funcref) (param $n i32)
        (table.fill $t (local.get $i) (local.get $r) (local.get $n))
      )
    
      (func (export "fill-abbrev") (param $i i32) (param $r funcref) (param $n i32)
        (table.fill $t (local.get $i) (local.get $r) (local.get $n))
      )
    
      (func (export "get") (param $i i32) (result funcref)
        (table.get $t (local.get $i))
      )
    )
    "#;

    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut i = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");

    let get = get_func!(i, "get");
    let fill = get_func!(i, "fill");
    let fill_abbrev = get_func!(i, "fill-abbrev");

    assert!(i.invoke_typed::<i32, RefFunc>(get, 1).unwrap().0.is_none());
    assert!(i.invoke_typed::<i32, RefFunc>(get, 2).unwrap().0.is_none());
    assert!(i.invoke_typed::<i32, RefFunc>(get, 3).unwrap().0.is_none());
    assert!(i.invoke_typed::<i32, RefFunc>(get, 4).unwrap().0.is_none());
    assert!(i.invoke_typed::<i32, RefFunc>(get, 5).unwrap().0.is_none());

    i.invoke_typed::<(i32, RefFunc, i32), ()>(fill, (2, RefFunc(Some(FuncAddr(1))), 3))
        .unwrap();

    assert!(i.invoke_typed::<i32, RefFunc>(get, 1).unwrap().0.is_none());
    assert_eq!(
        i.invoke_typed::<i32, RefFunc>(get, 2).unwrap().0.unwrap().0,
        1
    );
    assert_eq!(
        i.invoke_typed::<i32, RefFunc>(get, 3).unwrap().0.unwrap().0,
        1
    );
    assert_eq!(
        i.invoke_typed::<i32, RefFunc>(get, 4).unwrap().0.unwrap().0,
        1
    );
    assert!(i.invoke_typed::<i32, RefFunc>(get, 5).unwrap().0.is_none());

    i.invoke_typed::<(i32, RefFunc, i32), ()>(fill, (4, RefFunc(Some(FuncAddr(2))), 2))
        .unwrap();

    assert_eq!(
        i.invoke_typed::<i32, RefFunc>(get, 3).unwrap().0.unwrap().0,
        1
    );
    assert_eq!(
        i.invoke_typed::<i32, RefFunc>(get, 4).unwrap().0.unwrap().0,
        2
    );
    assert_eq!(
        i.invoke_typed::<i32, RefFunc>(get, 5).unwrap().0.unwrap().0,
        2
    );
    assert!(i.invoke_typed::<i32, RefFunc>(get, 6).unwrap().0.is_none());

    i.invoke_typed::<(i32, RefFunc, i32), ()>(fill, (4, RefFunc(Some(FuncAddr(3))), 0))
        .unwrap();

    assert_eq!(
        i.invoke_typed::<i32, RefFunc>(get, 3).unwrap().0.unwrap().0,
        1
    );
    assert_eq!(
        i.invoke_typed::<i32, RefFunc>(get, 4).unwrap().0.unwrap().0,
        2
    );
    assert_eq!(
        i.invoke_typed::<i32, RefFunc>(get, 5).unwrap().0.unwrap().0,
        2
    );

    i.invoke_typed::<(i32, RefFunc, i32), ()>(fill, (8, RefFunc(Some(FuncAddr(4))), 2))
        .unwrap();

    assert!(i.invoke_typed::<i32, RefFunc>(get, 7).unwrap().0.is_none());
    assert_eq!(
        i.invoke_typed::<i32, RefFunc>(get, 8).unwrap().0.unwrap().0,
        4
    );
    assert_eq!(
        i.invoke_typed::<i32, RefFunc>(get, 9).unwrap().0.unwrap().0,
        4
    );

    i.invoke_typed::<(i32, RefFunc, i32), ()>(fill_abbrev, (9, RefFunc(None), 1))
        .unwrap();
    assert_eq!(
        i.invoke_typed::<i32, RefFunc>(get, 8).unwrap().0.unwrap().0,
        4
    );
    assert!(i.invoke_typed::<i32, RefFunc>(get, 9).unwrap().0.is_none());

    i.invoke_typed::<(i32, RefFunc, i32), ()>(fill, (10, RefFunc(Some(FuncAddr(5))), 0))
        .unwrap();
    assert!(i.invoke_typed::<i32, RefFunc>(get, 9).unwrap().0.is_none());

    assert!(
        i.invoke_typed::<(i32, RefFunc, i32), ()>(fill, (8, RefFunc(Some(FuncAddr(6))), 3))
            .err()
            .unwrap()
            == RuntimeError::Trap(TrapError::TableOrElementAccessOutOfBounds)
    );

    assert!(i.invoke_typed::<i32, RefFunc>(get, 7).unwrap().0.is_none());
    assert_eq!(
        i.invoke_typed::<i32, RefFunc>(get, 8).unwrap().0.unwrap().0,
        4
    );
    assert!(i.invoke_typed::<i32, RefFunc>(get, 9).unwrap().0.is_none());

    assert!(
        i.invoke_typed::<(i32, RefFunc, i32), ()>(fill, (11, RefFunc(None), 0))
            .err()
            .unwrap()
            == RuntimeError::Trap(TrapError::TableOrElementAccessOutOfBounds)
    );

    assert!(
        i.invoke_typed::<(i32, RefFunc, i32), ()>(fill, (11, RefFunc(None), 10))
            .err()
            .unwrap()
            == RuntimeError::Trap(TrapError::TableOrElementAccessOutOfBounds)
    );
}
