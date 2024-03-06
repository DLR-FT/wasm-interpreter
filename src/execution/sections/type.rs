use crate::core::reader::types::FuncType;
use crate::core::reader::{WasmReadable, WasmReader};
use crate::execution::unwrap_validated::UnwrapValidatedExt;
use alloc::vec::Vec;

pub fn read_type_section(wasm: &mut WasmReader) -> Vec<FuncType> {
    let functypes = wasm
        .read_vec(|wasm| Ok(FuncType::read_unvalidated(wasm)))
        .unwrap_validated();
    functypes
}
