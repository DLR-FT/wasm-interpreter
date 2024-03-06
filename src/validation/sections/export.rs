use alloc::vec::Vec;

use crate::core::reader::section_header::{SectionHeader, SectionTy};
use crate::core::reader::types::export::Export;
use crate::core::reader::{WasmReadable, WasmReader};
use crate::execution::unwrap_validated::UnwrapValidatedExt;
use crate::Result;

pub fn read_export_section(
    wasm: &mut WasmReader,
    section_header: SectionHeader,
) -> Result<Vec<Export>> {
    assert_eq!(section_header.ty, SectionTy::Export);

    let exports = wasm.read_vec(|wasm| Export::read(wasm))?;
    debug!("Export section read: {:?}", &exports);
    Ok(exports)
}
