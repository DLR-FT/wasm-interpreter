use alloc::vec::Vec;

use crate::core::reader::section_header::{SectionHeader, SectionTy};
use crate::core::reader::types::{MemType, TableType};
use crate::core::reader::{WasmReadable, WasmReader};
use crate::Result;

pub fn read_memory_section(
    wasm: &mut WasmReader,
    section_header: SectionHeader,
) -> Result<Vec<MemType>> {
    assert_eq!(section_header.ty, SectionTy::Memory);

    let memories = wasm.read_vec(|wasm| MemType::read(wasm))?;

    debug!("Memory section read: {:?}", memories);
    Ok(memories)
}
