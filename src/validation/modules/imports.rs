use crate::{
    core::{
        decoding::reader::WasmReader,
        structure::modules::{
            imports::{Import, ImportDesc},
            indices::{IdxVec, TypeIdx},
        },
    },
    DecodingError, FuncType, GlobalType, MemType, TableType, ValidationError,
};

impl<'wasm> Import<'wasm> {
    pub fn read_and_validate(
        wasm: &mut WasmReader<'wasm>,
        c_types: &IdxVec<TypeIdx, FuncType>,
    ) -> Result<Self, ValidationError> {
        let module_name = wasm.read_name()?;
        let name = wasm.read_name()?;
        let desc = ImportDesc::read_and_validate(wasm, c_types)?;

        Ok(Self {
            module_name,
            name,
            desc,
        })
    }
}

impl ImportDesc {
    pub fn read_and_validate(
        wasm: &mut WasmReader,
        c_types: &IdxVec<TypeIdx, FuncType>,
    ) -> Result<Self, ValidationError> {
        let desc = match wasm.read_u8()? {
            0x00 => Self::Func(TypeIdx::read_and_validate(wasm, c_types)?),
            // https://webassembly.github.io/spec/core/binary/types.html#table-types
            0x01 => Self::Table(TableType::read_and_validate(wasm)?),
            0x02 => Self::Mem(MemType::read_and_validate(wasm)?),
            0x03 => Self::Global(GlobalType::read(wasm)?),
            other => return Err(DecodingError::MalformedImportDescDiscriminator(other).into()),
        };

        Ok(desc)
    }
}
