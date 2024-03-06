use crate::core::reader::types::GlobalType;
use crate::core::reader::{WasmReadable, WasmReader};
use crate::execution::unwrap_validated::UnwrapValidatedExt;
use alloc::vec::Vec;

pub fn read_global_section(wasm: &mut WasmReader) -> Vec<GlobalType> {
    wasm.read_vec(|wasm| Ok(GlobalType::read_unvalidated(wasm)))
        .unwrap_validated()
}
