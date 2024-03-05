use alloc::vec::Vec;

use crate::core::indices::TypeIdx;
use crate::core::reader::section_header::{SectionHeader, SectionTy};
use crate::core::reader::WasmReader;
use crate::Result;

pub fn read_function_section(
    wasm: &mut WasmReader,
    section_header: SectionHeader,
) -> Result<Vec<TypeIdx>> {
    assert_eq!(section_header.ty, SectionTy::Function);

    let typeidxs = wasm.read_vec(|wasm| wasm.read_var_u32().map(|u| u as usize))?;
    debug!("Function section read: {:?}", typeidxs);
    Ok(typeidxs)
}
