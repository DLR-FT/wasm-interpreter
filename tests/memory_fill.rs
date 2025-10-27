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

// use core::slice::SlicePattern;

use wasm::{validate, ExternVal, RuntimeInstance, DEFAULT_MODULE};

#[test_log::test]
fn memory_fill() {
    let w = r#"
    (module
        (memory (export "mem") 1)
        (func (export "fill")
            (memory.fill (i32.const 0) (i32.const 2777) (i32.const 100))
        )
    )
  "#;
    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let (mut i, module) = RuntimeInstance::new_with_default_module((), &validation_info)
        .expect("instantiation failed");

    let fill = i.get_function_by_name(DEFAULT_MODULE, "fill").unwrap();
    let ExternVal::Mem(mem) = i.store.instance_export(module, "mem").unwrap() else {
        panic!("expected memory")
    };

    i.invoke_typed::<(), ()>(&fill, ()).unwrap();

    let expected = [vec![217u8; 100], vec![0u8; 5]].concat();
    for (idx, expected_byte) in expected.into_iter().enumerate() {
        let mem_byte: u8 = i.store.mem_read(mem, idx as u32).unwrap();
        assert_eq!(
            mem_byte.to_ascii_lowercase(),
            expected_byte.to_ascii_lowercase()
        );
    }
}

// we need control flow implemented for any of these tests
#[ignore = "not yet implemented"]
#[test_log::test]
fn memory_fill_with_control_flow() {
    todo!()
}
