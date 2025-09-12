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
use wasm::{validate, RuntimeError, RuntimeInstance, TrapError, DEFAULT_MODULE};

macro_rules! get_func {
    ($instance:ident, $func_name:expr) => {
        &$instance
            .get_function_by_name(DEFAULT_MODULE, $func_name)
            .unwrap()
    };
}

macro_rules! is_specific_func {
    ($self:expr, $func_id:expr) => {
        match $self {
            Ref::Func(func_addr) => func_addr.addr == Some($func_id as usize),
            _ => unimplemented!(),
        }
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

    assert!(i
        .invoke_typed::<i32, FuncRefForInteropValue>(get, 1)
        .unwrap()
        .get_ref()
        .is_null());
    assert!(i
        .invoke_typed::<i32, FuncRefForInteropValue>(get, 2)
        .unwrap()
        .get_ref()
        .is_null());
    assert!(i
        .invoke_typed::<i32, FuncRefForInteropValue>(get, 3)
        .unwrap()
        .get_ref()
        .is_null());
    assert!(i
        .invoke_typed::<i32, FuncRefForInteropValue>(get, 4)
        .unwrap()
        .get_ref()
        .is_null());
    assert!(i
        .invoke_typed::<i32, FuncRefForInteropValue>(get, 5)
        .unwrap()
        .get_ref()
        .is_null());

    i.invoke_typed::<(i32, FuncRefForInteropValue, i32), ()>(
        fill,
        (
            2,
            FuncRefForInteropValue::new(Ref::Func(FuncAddr::new(Some(1)))).unwrap(),
            3,
        ),
    )
    .unwrap();

    assert!(i
        .invoke_typed::<i32, FuncRefForInteropValue>(get, 1)
        .unwrap()
        .get_ref()
        .is_null());
    assert!(is_specific_func!(
        i.invoke_typed::<i32, FuncRefForInteropValue>(get, 2)
            .unwrap()
            .get_ref(),
        1
    ));
    assert!(is_specific_func!(
        i.invoke_typed::<i32, FuncRefForInteropValue>(get, 3)
            .unwrap()
            .get_ref(),
        1
    ));
    assert!(is_specific_func!(
        i.invoke_typed::<i32, FuncRefForInteropValue>(get, 4)
            .unwrap()
            .get_ref(),
        1
    ));
    assert!(i
        .invoke_typed::<i32, FuncRefForInteropValue>(get, 5)
        .unwrap()
        .get_ref()
        .is_null());

    i.invoke_typed::<(i32, FuncRefForInteropValue, i32), ()>(
        fill,
        (
            4,
            FuncRefForInteropValue::new(Ref::Func(FuncAddr::new(Some(2)))).unwrap(),
            2,
        ),
    )
    .unwrap();

    assert!(is_specific_func!(
        i.invoke_typed::<i32, FuncRefForInteropValue>(get, 3)
            .unwrap()
            .get_ref(),
        1
    ));
    assert!(is_specific_func!(
        i.invoke_typed::<i32, FuncRefForInteropValue>(get, 4)
            .unwrap()
            .get_ref(),
        2
    ));
    assert!(is_specific_func!(
        i.invoke_typed::<i32, FuncRefForInteropValue>(get, 5)
            .unwrap()
            .get_ref(),
        2
    ));
    assert!(i
        .invoke_typed::<i32, FuncRefForInteropValue>(get, 6)
        .unwrap()
        .get_ref()
        .is_null());

    i.invoke_typed::<(i32, FuncRefForInteropValue, i32), ()>(
        fill,
        (
            4,
            FuncRefForInteropValue::new(Ref::Func(FuncAddr::new(Some(3)))).unwrap(),
            0,
        ),
    )
    .unwrap();

    assert!(is_specific_func!(
        i.invoke_typed::<i32, FuncRefForInteropValue>(get, 3)
            .unwrap()
            .get_ref(),
        1
    ));
    assert!(is_specific_func!(
        i.invoke_typed::<i32, FuncRefForInteropValue>(get, 4)
            .unwrap()
            .get_ref(),
        2
    ));
    assert!(is_specific_func!(
        i.invoke_typed::<i32, FuncRefForInteropValue>(get, 5)
            .unwrap()
            .get_ref(),
        2
    ));

    i.invoke_typed::<(i32, FuncRefForInteropValue, i32), ()>(
        fill,
        (
            8,
            FuncRefForInteropValue::new(Ref::Func(FuncAddr::new(Some(4)))).unwrap(),
            2,
        ),
    )
    .unwrap();

    assert!(i
        .invoke_typed::<i32, FuncRefForInteropValue>(get, 7)
        .unwrap()
        .get_ref()
        .is_null());
    assert!(is_specific_func!(
        i.invoke_typed::<i32, FuncRefForInteropValue>(get, 8)
            .unwrap()
            .get_ref(),
        4
    ));
    assert!(is_specific_func!(
        i.invoke_typed::<i32, FuncRefForInteropValue>(get, 9)
            .unwrap()
            .get_ref(),
        4
    ));

    i.invoke_typed::<(i32, FuncRefForInteropValue, i32), ()>(
        fill_abbrev,
        (
            9,
            FuncRefForInteropValue::new(Ref::Func(FuncAddr::null())).unwrap(),
            1,
        ),
    )
    .unwrap();
    assert!(is_specific_func!(
        i.invoke_typed::<i32, FuncRefForInteropValue>(get, 8)
            .unwrap()
            .get_ref(),
        4
    ));
    assert!(i
        .invoke_typed::<i32, FuncRefForInteropValue>(get, 9)
        .unwrap()
        .get_ref()
        .is_null());

    i.invoke_typed::<(i32, FuncRefForInteropValue, i32), ()>(
        fill,
        (
            10,
            FuncRefForInteropValue::new(Ref::Func(FuncAddr::new(Some(5)))).unwrap(),
            0,
        ),
    )
    .unwrap();
    assert!(i
        .invoke_typed::<i32, FuncRefForInteropValue>(get, 9)
        .unwrap()
        .get_ref()
        .is_null());

    assert!(
        i.invoke_typed::<(i32, FuncRefForInteropValue, i32), ()>(
            fill,
            (
                8,
                FuncRefForInteropValue::new(Ref::Func(FuncAddr::new(Some(6)))).unwrap(),
                3
            )
        )
        .err()
        .unwrap()
            == RuntimeError::Trap(TrapError::TableOrElementAccessOutOfBounds)
    );

    assert!(i
        .invoke_typed::<i32, FuncRefForInteropValue>(get, 7)
        .unwrap()
        .get_ref()
        .is_null());
    assert!(is_specific_func!(
        i.invoke_typed::<i32, FuncRefForInteropValue>(get, 8)
            .unwrap()
            .get_ref(),
        4
    ));
    assert!(i
        .invoke_typed::<i32, FuncRefForInteropValue>(get, 9)
        .unwrap()
        .get_ref()
        .is_null());

    assert!(
        i.invoke_typed::<(i32, FuncRefForInteropValue, i32), ()>(
            fill,
            (
                11,
                FuncRefForInteropValue::new(Ref::Func(FuncAddr::null())).unwrap(),
                0
            )
        )
        .err()
        .unwrap()
            == RuntimeError::Trap(TrapError::TableOrElementAccessOutOfBounds)
    );

    assert!(
        i.invoke_typed::<(i32, FuncRefForInteropValue, i32), ()>(
            fill,
            (
                11,
                FuncRefForInteropValue::new(Ref::Func(FuncAddr::null())).unwrap(),
                10
            )
        )
        .err()
        .unwrap()
            == RuntimeError::Trap(TrapError::TableOrElementAccessOutOfBounds)
    );
}
