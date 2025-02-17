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

    pub functions: Vec<usize>,
    pub functions_offset: usize,
    pub imported_functions_len: usize,

    pub memories: Vec<usize>,
    pub memories_offset: usize,
    pub imported_memories_len: usize,

    pub globals: Vec<usize>,
    pub globals_offset: usize,
    pub imported_globals_len: usize,

    pub tables: Vec<usize>,
    pub tables_offset: usize,
    pub imported_tables_len: usize,

    pub data: Vec<usize>,
    pub data_offset: usize,

    pub elements: Vec<usize>,
    pub elements_offset: usize,

    pub passive_element_indexes: Vec<usize>,
    // pub exports: Vec<usize>,
}
