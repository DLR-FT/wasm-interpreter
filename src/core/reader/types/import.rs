use alloc::borrow::ToOwned;
use alloc::string::String;

use crate::core::indices::{CTypes, TypeIdx};
use crate::core::reader::WasmReader;
use crate::{ValidationError, ValidationInfo};

use super::global::GlobalType;
use super::{ExternType, MemType, TableType};

#[derive(Debug, Clone)]
pub struct Import {
    #[allow(warnings)]
    pub module_name: String,
    #[allow(warnings)]
    pub name: String,
    #[allow(warnings)]
    pub desc: ImportDesc,
}

impl Import {
    pub fn read(wasm: &mut WasmReader, c_types: &CTypes) -> Result<Self, ValidationError> {
        let module_name = wasm.read_name()?.to_owned();
        let name = wasm.read_name()?.to_owned();
        let desc = ImportDesc::read(wasm, c_types)?;

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
    pub fn read(wasm: &mut WasmReader, c_types: &CTypes) -> Result<Self, ValidationError> {
        let desc = match wasm.read_u8()? {
            0x00 => Self::Func(TypeIdx::read_and_validate(wasm, c_types)?),
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
    ///<https://webassembly.github.io/spec/core/valid/modules.html#imports>
    ///
    /// # Safety
    /// The given [`ValidationInfo`] must be created from the same Wasm bytecode as `self` was read from.
    pub unsafe fn extern_type(&self, validation_info: &ValidationInfo) -> ExternType {
        match self {
            ImportDesc::Func(type_idx) => {
                // unlike ExportDescs, these directly refer to the types section
                // since a corresponding function entry in function section or body
                // in code section does not exist for these
                let func_type = validation_info.c_types.get(*type_idx);
                // TODO ugly clone that should disappear when types are directly parsed from bytecode instead of vector copies
                ExternType::Func(func_type.clone())
            }
            ImportDesc::Table(ty) => ExternType::Table(*ty),
            ImportDesc::Mem(ty) => ExternType::Mem(*ty),
            ImportDesc::Global(ty) => ExternType::Global(*ty),
        }
    }
}
