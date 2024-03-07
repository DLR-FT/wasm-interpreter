use alloc::vec::Vec;

use crate::core::reader::{WasmReadable, WasmReader};
use crate::core::reader::section_header::{SectionHeader, SectionTy};
use crate::core::reader::types::GlobalType;
use crate::Result;

pub fn read_global_section(
    wasm: &mut WasmReader,
    section_header: SectionHeader,
) -> Result<Vec<GlobalType>> {
    assert_eq!(section_header.ty, SectionTy::Global);

    let globals = wasm.read_vec(|wasm| GlobalType::read(wasm))?;

    debug!("Global section read: {:?}", globals);
    Ok(globals)
}
