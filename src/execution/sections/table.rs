use crate::core::reader::types::TableType;
use crate::core::reader::{WasmReadable, WasmReader};
use crate::execution::unwrap_validated::UnwrapValidatedExt;
use alloc::vec::Vec;

pub fn read_table_section(wasm: &mut WasmReader) -> Vec<TableType> {
    wasm.read_vec(|wasm| Ok(TableType::read_unvalidated(wasm)))
        .unwrap_validated()
}
