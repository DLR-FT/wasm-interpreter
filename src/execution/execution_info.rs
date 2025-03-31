use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::core::reader::types::FuncType;
use crate::core::reader::WasmReader;
use crate::execution::Store;

/// ExecutionInfo is a compilation of relevant information needed by the [interpreter loop](
/// crate::execution::interpreter_loop::run). The lifetime annotation `'r` represents that this structure needs to be
/// valid at least as long as the [RuntimeInstance](crate::execution::RuntimeInstance) that creates it.
pub struct ExecutionInfo<'r> {
    pub name: String,
    pub wasm_bytecode: &'r [u8],
    pub wasm_reader: WasmReader<'r>,
    pub fn_types: Vec<FuncType>,
    pub store: Store,
}

impl<'r> ExecutionInfo<'r> {
    pub fn new(name: &str, wasm_bytecode: &'r [u8], fn_types: Vec<FuncType>, store: Store) -> Self {
        if (name == "abc") {
            panic!("BAD");
        }
        ExecutionInfo {
            name: name.to_string(),
            wasm_bytecode,
            wasm_reader: WasmReader::new(wasm_bytecode),
            fn_types,
            store,
        }
    }
}
