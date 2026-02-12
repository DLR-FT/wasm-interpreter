use alloc::borrow::ToOwned;
use alloc::string::String;

use crate::core::indices::{IdxVec, TypeIdx};
use crate::core::reader::types::FuncType;
use crate::core::reader::WasmReader;
use crate::{ValidationError, ValidationInfo};

use super::global::GlobalType;
use super::{ExternType, MemType, TableType};

#[derive(Debug, Clone)]
pub struct Import {
    pub module_name: String,
    pub name: String,
    pub desc: ImportDesc,
}

impl Import {
    pub fn read_and_validate(
        wasm: &mut WasmReader,
        c_types: &IdxVec<TypeIdx, FuncType>,
    ) -> Result<Self, ValidationError> {
        let module_name = wasm.read_name()?.to_owned();
        let name = wasm.read_name()?.to_owned();
        let desc = ImportDesc::read_and_validate(wasm, c_types)?;

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
    pub fn read_and_validate(
        wasm: &mut WasmReader,
        c_types: &IdxVec<TypeIdx, FuncType>,
    ) -> Result<Self, ValidationError> {
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
    /// # Safety
    ///
    /// The caller must ensure that `self` comes from the same
    /// [`ValidationInfo`] that is passed as an argument here.
    pub unsafe fn extern_type(&self, validation_info: &ValidationInfo) -> ExternType {
        match self {
            ImportDesc::Func(type_idx) => {
                // unlike ExportDescs, these directly refer to the types section
                // since a corresponding function entry in function section or body
                // in code section does not exist for these

                // SAFETY: The caller ensures that the current `ImportDesc`
                // comes from the same `ValidationInfo`. Because all type
                // indices contained by a `ValidationInfo` must always be valid,
                // this is safe.
                let func_type = unsafe { validation_info.types.get(*type_idx) };
                // TODO ugly clone that should disappear when types are directly parsed from bytecode instead of vector copies
                ExternType::Func(func_type.clone())
            }
            ImportDesc::Table(ty) => ExternType::Table(*ty),
            ImportDesc::Mem(ty) => ExternType::Mem(*ty),
            ImportDesc::Global(ty) => ExternType::Global(*ty),
        }
    }
}
