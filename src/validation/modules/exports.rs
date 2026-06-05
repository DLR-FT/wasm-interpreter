use crate::{
    core::{
        decoding::reader::WasmReader,
        structure::modules::{
            exports::{Export, ExportDesc},
            globals::Global,
            indices::{FuncIdx, GlobalIdx, IdxVec, MemIdx, TableIdx, TypeIdx},
        },
    },
    DecodingError, MemType, TableType, ValidationError,
};

impl<'wasm> Export<'wasm> {
    pub fn read_and_validate(
        wasm: &mut WasmReader<'wasm>,
        c_funcs: &IdxVec<FuncIdx, TypeIdx>,
        c_tables: &IdxVec<TableIdx, TableType>,
        c_mems: &IdxVec<MemIdx, MemType>,
        c_globals: &IdxVec<GlobalIdx, Global>,
    ) -> Result<Self, ValidationError> {
        let name = wasm.read_name()?;
        let desc = ExportDesc::read_and_validate(wasm, c_funcs, c_tables, c_mems, c_globals)?;
        Ok(Export { name, desc })
    }
}

impl ExportDesc {
    pub fn read_and_validate(
        wasm: &mut WasmReader,
        c_functions: &IdxVec<FuncIdx, TypeIdx>,
        c_tables: &IdxVec<TableIdx, TableType>,
        c_mems: &IdxVec<MemIdx, MemType>,
        c_globals: &IdxVec<GlobalIdx, Global>,
    ) -> Result<Self, ValidationError> {
        let desc_id = wasm.read_u8()?;

        let desc = match desc_id {
            0x00 => ExportDesc::Func(FuncIdx::read_and_validate(wasm, c_functions)?),
            0x01 => ExportDesc::Table(TableIdx::read_and_validate(wasm, c_tables)?),
            0x02 => ExportDesc::Mem(MemIdx::read_and_validate(wasm, c_mems)?),
            0x03 => ExportDesc::Global(GlobalIdx::read_and_validate(wasm, c_globals)?),
            other => return Err(DecodingError::MalformedExportDescDiscriminator(other).into()),
        };
        Ok(desc)
    }
}
