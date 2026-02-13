use crate::core::indices::TypeIdx;
use crate::core::reader::WasmReader;
use crate::{ValidationError, ValidationInfo};

use super::global::GlobalType;
use super::{ExternType, MemType, TableType};

#[derive(Debug, Clone)]
pub struct Import<'wasm> {
    pub module_name: &'wasm str,
    pub name: &'wasm str,
    pub desc: ImportDesc,
}

impl<'wasm> Import<'wasm> {
    pub fn read(wasm: &mut WasmReader<'wasm>) -> Result<Self, ValidationError> {
        let module_name = wasm.read_name()?;
        let name = wasm.read_name()?;
        let desc = ImportDesc::read(wasm)?;

        Ok(Self {
            module_name,
            name,
            desc,
        })
    }
}

#[derive(Debug, Clone)]
pub enum ImportDesc {
    Func(TypeIdx),
    Table(TableType),
    Mem(MemType),
    Global(GlobalType),
}

impl ImportDesc {
    pub fn read(wasm: &mut WasmReader) -> Result<Self, ValidationError> {
        let desc = match wasm.read_u8()? {
            0x00 => Self::Func(wasm.read_var_u32()? as TypeIdx),
            // https://webassembly.github.io/spec/core/binary/types.html#table-types
            0x01 => Self::Table(TableType::read(wasm)?),
            0x02 => Self::Mem(MemType::read(wasm)?),
            0x03 => Self::Global(GlobalType::read(wasm)?),
            other => return Err(ValidationError::MalformedImportDescDiscriminator(other)),
        };

        Ok(desc)
    }
}

impl ImportDesc {
    /// returns the external type of `self` according to typing relation,
    /// taking `validation_info` as validation context C
    ///
    /// Note: This method may panic if self does not come from the given [`ValidationInfo`].
    ///<https://webassembly.github.io/spec/core/valid/modules.html#imports>
    pub fn extern_type(&self, validation_info: &ValidationInfo) -> ExternType {
        match self {
            ImportDesc::Func(type_idx) => {
                // unlike ExportDescs, these directly refer to the types section
                // since a corresponding function entry in function section or body
                // in code section does not exist for these
                let func_type = validation_info
                    .types
                    .get(*type_idx)
                    .expect("type index of import descs to always be valid if the validation info is correct");
                // TODO ugly clone that should disappear when types are directly parsed from bytecode instead of vector copies
                ExternType::Func(func_type.clone())
            }
            ImportDesc::Table(ty) => ExternType::Table(*ty),
            ImportDesc::Mem(ty) => ExternType::Mem(*ty),
            ImportDesc::Global(ty) => ExternType::Global(*ty),
        }
    }
}
