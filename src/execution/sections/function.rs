use crate::core::indices::TypeIdx;
use crate::core::reader::WasmReader;
use crate::execution::unwrap_validated::UnwrapValidatedExt;
use alloc::vec::Vec;

pub fn read_function_section(wasm: &mut WasmReader) -> Vec<TypeIdx> {
    let typeidxs = wasm
        .read_vec(|wasm| Ok(wasm.read_var_u32().unwrap_validated() as usize))
        .unwrap_validated();

    typeidxs
}
