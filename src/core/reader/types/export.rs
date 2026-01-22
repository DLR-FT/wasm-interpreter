use alloc::borrow::ToOwned;
use alloc::string::String;

use crate::core::indices::{ExtendedIdxVec, FuncIdx, GlobalIdx, MemIdx, TableIdx, TypeIdx};
use crate::core::reader::types::import::ImportDesc;
use crate::core::reader::WasmReader;
use crate::{MemType, TableType, ValidationError, ValidationInfo};

use super::ExternType;

#[derive(Debug, Clone)]
pub struct Export {
    pub name: String,
    pub desc: ExportDesc,
}

impl Export {
    pub fn read_and_validate(
        wasm: &mut WasmReader,
        c_funcs: &ExtendedIdxVec<FuncIdx, TypeIdx>,
        c_tables: &ExtendedIdxVec<TableIdx, TableType>,
        c_mems: &ExtendedIdxVec<MemIdx, MemType>,
    ) -> Result<Self, ValidationError> {
        let name = wasm.read_name()?.to_owned();
        let desc = ExportDesc::read_and_validate(wasm, c_funcs, c_tables, c_mems)?;
        Ok(Export { name, desc })
    }
}

#[derive(Debug, Clone)]
// TODO: change enum labels from FuncIdx -> Func
pub enum ExportDesc {
    Func(FuncIdx),
    Table(TableIdx),
    Mem(MemIdx),
    Global(GlobalIdx),
}

impl ExportDesc {
    /// returns the external type of `self` according to typing relation,
    /// taking `validation_info` as validation context C
    ///
    /// # Safety
    ///
    /// The caller must ensure that `self` comes from the same
    /// [`ValidationInfo`] that is passed as an argument here.
    #[allow(unused)] // reason = "this function is analogous to ImportDesc::extern_type, however it is not yet clear if it is needed in the future"
    pub unsafe fn extern_type(&self, validation_info: &ValidationInfo) -> ExternType {
        // TODO clean up logic for checking if an exported definition is an
        // import
        match self {
            ExportDesc::Func(func_idx) => {
                // SAFETY: The caller ensures that the current `ExportDesc`
                // comes from the same `ValidationInfo` that is passed into the
                // current function. Therefore, the function index stored in
                // `self` must be valid in the given `ValidationInfo`.
                let type_idx = unsafe { validation_info.functions.get(*func_idx) };
                // SAFETY: The type index was just read from the passed
                // `ValidationInfo`.  Because the `ValidationInfo` struct
                // guarantees that all indices contained in it are valid for all
                // other `IdxVec` vectors in it, this is sound.
                let func_type = unsafe { validation_info.types.get(*type_idx) };
                // TODO ugly clone that should disappear when types are directly parsed from bytecode instead of vector copies
                ExternType::Func(func_type.clone())
            }
            ExportDesc::Table(table_idx) => {
                // SAFETY: The caller ensures that the current `ExportDesc`
                // comes from the same `ValidationInfo` that is passed into the
                // current function. Therefore, the table index stored in `self`
                // must be valid in the given `ValidationInfo`.
                let table_type = unsafe { validation_info.tables.get(*table_idx) };

                ExternType::Table(*table_type)
            }
            ExportDesc::Mem(mem_idx) => {
                // SAFETY: The caller ensures that the current `ExportDesc`
                // comes from the same `ValidationInfo` that is passed into the
                // current function. Therefore, the memory index stored in
                // `self` must be valid in the given `ValidationInfo`.
                let mem_type = unsafe { validation_info.memories.get(*mem_idx) };

                ExternType::Mem(*mem_type)
            }
            ExportDesc::Global(global_idx) => {
                let global_type =
                    match global_idx.checked_sub(validation_info.imports_length.imported_globals) {
                        Some(local_global_idx) => {
                            validation_info.globals.get(local_global_idx).unwrap().ty
                        }
                        None => validation_info
                            .imports
                            .iter()
                            .filter_map(|import| match import.desc {
                                ImportDesc::Global(global_type) => Some(global_type),
                                _ => None,
                            })
                            .nth(*global_idx)
                            .unwrap(),
                    };

                ExternType::Global(global_type)
            }
        }
    }
}

impl ExportDesc {
    pub fn read_and_validate(
        wasm: &mut WasmReader,
        c_functions: &ExtendedIdxVec<FuncIdx, TypeIdx>,
        c_tables: &ExtendedIdxVec<TableIdx, TableType>,
        c_mems: &ExtendedIdxVec<MemIdx, MemType>,
    ) -> Result<Self, ValidationError> {
        let desc_id = wasm.read_u8()?;

        let desc = match desc_id {
            0x00 => ExportDesc::Func(FuncIdx::read_and_validate(wasm, c_functions)?),
            0x01 => ExportDesc::Table(TableIdx::read_and_validate(wasm, c_tables)?),
            0x02 => ExportDesc::Mem(MemIdx::read_and_validate(wasm, c_mems)?),
            0x03 => {
                let desc_idx = wasm.read_var_u32()? as usize;
                ExportDesc::Global(desc_idx)
            }
            other => return Err(ValidationError::MalformedExportDescDiscriminator(other)),
        };
        Ok(desc)
    }
}
