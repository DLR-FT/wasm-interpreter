use crate::core::indices::FuncIdx;
use crate::core::reader::WasmReader;
use crate::core::reader::section_header::{SectionHeader, SectionTy};
use crate::Result;

pub fn read_start_section(wasm: &mut WasmReader, section_header: SectionHeader) -> Result<FuncIdx> {
    assert_eq!(section_header.ty, SectionTy::Start);

    let start = wasm.read_var_u32()? as FuncIdx;

    debug!("Start section read: {:?}", start);
    Ok(start)
}
