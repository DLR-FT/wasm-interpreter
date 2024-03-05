use alloc::vec::Vec;

use crate::core::reader::{WasmReadable, WasmReader};
use crate::core::reader::section_header::{SectionHeader, SectionTy};
use crate::core::reader::types::FuncType;
use crate::Result;

pub fn validate_type_section(
    wasm: &mut WasmReader,
    section_header: SectionHeader,
) -> Result<Vec<FuncType>> {
    assert_eq!(section_header.ty, SectionTy::Type);

    let functypes = wasm.read_vec(|wasm| FuncType::read(wasm))?;
    debug!("Type section read: {:?}", &functypes);
    Ok(functypes)
}
