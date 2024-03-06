use alloc::vec::Vec;

use crate::core::reader::{WasmReadable, WasmReader};
use crate::core::reader::types::export::Export;
use crate::execution::unwrap_validated::UnwrapValidatedExt;

pub fn read_export_section(wasm: &mut WasmReader) -> Vec<Export> {
    wasm.read_vec(|wasm| Ok(Export::read_unvalidated(wasm)))
        .unwrap_validated()
}
