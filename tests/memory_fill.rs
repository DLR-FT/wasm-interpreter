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

use wasm::{validate, RuntimeInstance, DEFAULT_MODULE};

macro_rules! get_func {
    ($instance:ident, $func_name:expr) => {
        &$instance
            .get_function_by_name(DEFAULT_MODULE, $func_name)
            .unwrap()
    };
}

#[test_log::test]
fn memory_fill() {
    let w = r#"
    (module
        (memory 1)
        (func (export "fill")
            (memory.fill (i32.const 0) (i32.const 2777) (i32.const 100))
        )
    )
  "#;
    let wasm_bytes = wat::parse_str(w).unwrap();
    let validation_info = validate(&wasm_bytes).unwrap();
    let mut i = RuntimeInstance::new(&validation_info).expect("instantiation failed");

    let fill = get_func!(i, "fill");
    i.invoke_typed::<(), ()>(fill, ()).unwrap();
    let mem_inst = &i.store.memories[0];

    let expected = [vec![217u8; 100], vec![0u8; 5]].concat();
    for (idx, expected_byte) in expected.into_iter().enumerate() {
        let mem_byte: u8 = mem_inst.mem.load(idx).unwrap();
        assert!(mem_byte.eq_ignore_ascii_case(&expected_byte));
    }
}

// we need control flow implemented for any of these tests
#[ignore = "not yet implemented"]
#[test_log::test]
fn memory_fill_with_control_flow() {
    todo!()
}
