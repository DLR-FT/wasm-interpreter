use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::core::reader::types::FuncType;
use crate::core::reader::WasmReader;
use crate::core::sidetable::Sidetable;
use crate::execution::Store;

/// ExecutionInfo is a compilation of relevant information needed by the [interpreter loop](
/// crate::execution::interpreter_loop::run). The lifetime annotation `'r` represents that this structure needs to be
/// valid at least as long as the [RuntimeInstance](crate::execution::RuntimeInstance) that creates it.
pub struct ExecutionInfo {
    pub name: String,
    pub fn_types: Vec<FuncType>,
    pub store: Store,
}

impl ExecutionInfo {
    pub fn new(name: &str, fn_types: Vec<FuncType>, store: Store) -> Self {
        ExecutionInfo {
            name: name.to_string(),
            fn_types,
            store,
        }
    }
}

pub struct StateData<'r> {
    pub wasm_bytecode: &'r [u8],
    pub wasm_reader: WasmReader<'r>,
    //TODO: figure out the lifetime of a sidetable ref, replace this with a ref
    pub sidetable: Sidetable,
}

impl<'r> StateData<'r> {
    pub fn new(wasm_bytecode: &'r [u8], sidetable: Sidetable) -> Self {
        StateData {
            wasm_bytecode,
            wasm_reader: WasmReader::new(wasm_bytecode),
            sidetable,
        }
    }
}
