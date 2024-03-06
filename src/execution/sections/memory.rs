use crate::core::reader::types::MemType;
use crate::core::reader::{WasmReadable, WasmReader};
use crate::execution::unwrap_validated::UnwrapValidatedExt;
use alloc::vec::Vec;

pub fn read_memory_section(wasm: &mut WasmReader) -> Vec<MemType> {
    wasm.read_vec(|wasm| Ok(MemType::read_unvalidated(wasm)))
        .unwrap_validated()
}
