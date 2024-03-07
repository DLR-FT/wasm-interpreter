use alloc::vec::Vec;

use crate::core::reader::{WasmReadable, WasmReader};
use crate::core::reader::section_header::{SectionHeader, SectionTy};
use crate::core::reader::types::TableType;
use crate::Result;

pub fn read_table_section(
    wasm: &mut WasmReader,
    section_header: SectionHeader,
) -> Result<Vec<TableType>> {
    assert_eq!(section_header.ty, SectionTy::Table);

    let tables = wasm.read_vec(|wasm| TableType::read(wasm))?;

    debug!("Table section read: {:?}", tables);
    Ok(tables)
}
