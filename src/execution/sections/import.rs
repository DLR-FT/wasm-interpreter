use alloc::vec::Vec;

use crate::core::reader::types::import::Import;
use crate::core::reader::{WasmReadable, WasmReader};
use crate::execution::unwrap_validated::UnwrapValidatedExt;

pub fn read_import_section(wasm: &mut WasmReader) -> Vec<Import> {
    wasm.read_vec(|wasm| Ok(Import::read_unvalidated(wasm)))
        .unwrap_validated()
}
