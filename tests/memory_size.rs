use log::info;
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
use wasm::{validate, RuntimeInstance, DEFAULT_MODULE};

#[test_log::test]
fn memory_size_1() {
    let w = r#"
(module
  (memory 0)
  (func (export "size") (result i32) (memory.size))
  (func (export "grow") (param $sz i32) (drop (memory.grow (local.get $sz))))
)
  "#;
    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let (mut i, _module) = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");

    let size = i.get_function_by_name(DEFAULT_MODULE, "size").unwrap();
    let grow = i.get_function_by_name(DEFAULT_MODULE, "grow").unwrap();

    assert_eq!(i.invoke_typed(size, ()), Ok(0));
    assert_eq!(i.invoke_typed(grow, 1), Ok(()));
    assert_eq!(i.invoke_typed(size, ()), Ok(1));
    assert_eq!(i.invoke_typed(grow, 4), Ok(()));
    assert_eq!(i.invoke_typed(size, ()), Ok(5));
    assert_eq!(i.invoke_typed(grow, 0), Ok(()));
    assert_eq!(i.invoke_typed(size, ()), Ok(5));
}

#[test_log::test]
fn memory_size_2() {
    let w = r#"
(module
  (memory 1)
  (func (export "size") (result i32) (memory.size))
  (func (export "grow") (param $sz i32) (drop (memory.grow (local.get $sz))))
)
  "#;
    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let (mut i, _module) = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");

    let size = i.get_function_by_name(DEFAULT_MODULE, "size").unwrap();
    let grow = i.get_function_by_name(DEFAULT_MODULE, "grow").unwrap();

    assert_eq!(i.invoke_typed(size, ()), Ok(1));
    assert_eq!(i.invoke_typed(grow, 1), Ok(()));
    assert_eq!(i.invoke_typed(size, ()), Ok(2));
    assert_eq!(i.invoke_typed(grow, 4), Ok(()));
    assert_eq!(i.invoke_typed(size, ()), Ok(6));
    assert_eq!(i.invoke_typed(grow, 0), Ok(()));
    assert_eq!(i.invoke_typed(size, ()), Ok(6));
}

#[test_log::test]
fn memory_size_3() {
    let w = r#"
(module
  (memory 0 2)
  (func (export "size") (result i32) (memory.size))
  (func (export "grow") (param $sz i32) (drop (memory.grow (local.get $sz))))
)
"#;
    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let (mut i, _module) = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");

    let size = i.get_function_by_name(DEFAULT_MODULE, "size").unwrap();
    let grow = i.get_function_by_name(DEFAULT_MODULE, "grow").unwrap();

    assert_eq!(i.invoke_typed(size, ()), Ok(0));
    assert_eq!(i.invoke_typed(grow, 3), Ok(()));
    assert_eq!(i.invoke_typed(size, ()), Ok(0));
    assert_eq!(i.invoke_typed(grow, 1), Ok(()));
    assert_eq!(i.invoke_typed(size, ()), Ok(1));
    assert_eq!(i.invoke_typed(grow, 0), Ok(()));
    assert_eq!(i.invoke_typed(size, ()), Ok(1));
    assert_eq!(i.invoke_typed(grow, 4), Ok(()));
    assert_eq!(i.invoke_typed(size, ()), Ok(1));
    assert_eq!(i.invoke_typed(grow, 1), Ok(()));
    assert_eq!(i.invoke_typed(size, ()), Ok(2));
}

#[test_log::test]
fn memory_size_4() {
    let w = r#"
(module
  (memory 3 8)
  (func (export "size") (result i32) (memory.size))
  (func (export "grow") (param $sz i32) (drop (memory.grow (local.get $sz))))
)
"#;
    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let (mut i, _module) = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");

    let size = i.get_function_by_name(DEFAULT_MODULE, "size").unwrap();
    let grow = i.get_function_by_name(DEFAULT_MODULE, "grow").unwrap();

    assert_eq!(i.invoke_typed(size, ()), Ok(3));
    assert_eq!(i.invoke_typed(grow, 1), Ok(()));
    assert_eq!(i.invoke_typed(size, ()), Ok(4));
    assert_eq!(i.invoke_typed(grow, 3), Ok(()));
    assert_eq!(i.invoke_typed(size, ()), Ok(7));
    assert_eq!(i.invoke_typed(grow, 0), Ok(()));
    assert_eq!(i.invoke_typed(size, ()), Ok(7));
    assert_eq!(i.invoke_typed(grow, 2), Ok(()));
    assert_eq!(i.invoke_typed(size, ()), Ok(7));
    assert_eq!(i.invoke_typed(grow, 1), Ok(()));
    assert_eq!(i.invoke_typed(size, ()), Ok(8));
}

#[test_log::test]
fn memory_size_5() {
    let w = r#"
  (module
    (memory 1)
    (func $type-result-i32-vs-empty
      (memory.size)
    )
  )
  "#;
    let wasm_bytes = wat::parse_str(w).unwrap();
    let mut i = 0;
    let mut info_str = String::new();
    for byte in wasm_bytes.iter() {
        info_str.push_str(&format!("{byte:#04X} "));
        i += 1;
        if i % 8 == 0 {
            i = 0;
            info!("{info_str}");
        }
    }
    let validation_info = validate(&wasm_bytes);
    assert_eq!(
        validation_info.err(),
        Some(wasm::ValidationError::EndInvalidValueStack)
    );
}

#[test_log::test]
fn memory_size_6() {
    let w = r#"
  (module
    (memory 1)
    (func $type-result-i32-vs-f32 (result f32)
      (memory.size)
    )
  )
  "#;
    let wasm_bytes = wat::parse_str(w).unwrap();
    let mut i = 0;
    let mut info_str = String::new();
    for byte in wasm_bytes.iter() {
        info_str.push_str(&format!("{byte:#04X} "));
        i += 1;
        if i % 8 == 0 {
            i = 0;
            info!("{info_str}");
        }
    }
    let validation_info = validate(&wasm_bytes);
    assert_eq!(
        validation_info.err(),
        Some(wasm::ValidationError::EndInvalidValueStack)
    );
}
