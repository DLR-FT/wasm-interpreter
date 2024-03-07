use crate::core::reader::section_header::{SectionHeader, SectionTy};
use crate::core::reader::types::import::Import;
use crate::core::reader::{WasmReadable, WasmReader};
use crate::Result;
use alloc::vec::Vec;

pub fn validate_import_section(
    wasm: &mut WasmReader,
    section_header: SectionHeader,
) -> Result<Vec<Import>> {
    assert_eq!(section_header.ty, SectionTy::Import);

    let imports = wasm.read_vec(|wasm| Import::read(wasm))?;
    debug!("Import section read: {:?}", &imports);
    Ok(imports)
}
