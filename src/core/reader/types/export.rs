use alloc::borrow::ToOwned;
use alloc::string::String;

use crate::core::indices::{FuncIdx, GlobalIdx, MemIdx, TableIdx};
use crate::core::reader::types::import::ImportDesc;
use crate::core::reader::WasmReader;
use crate::core::utils::ToUsizeExt;
use crate::{ValidationError, ValidationInfo};

use super::ExternType;

#[derive(Debug, Clone)]
pub struct Export {
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    pub desc: ExportDesc,
}

impl Export {
    pub fn read(wasm: &mut WasmReader) -> Result<Self, ValidationError> {
        let name = wasm.read_name()?.to_owned();
        let desc = ExportDesc::read(wasm)?;
        Ok(Export { name, desc })
    }
}

#[derive(Debug, Clone)]
#[allow(clippy::all)]
// TODO: change enum labels from FuncIdx -> Func
pub enum ExportDesc {
    #[allow(warnings)]
    FuncIdx(FuncIdx),
    #[allow(warnings)]
    TableIdx(TableIdx),
    #[allow(warnings)]
    MemIdx(MemIdx),
    #[allow(warnings)]
    GlobalIdx(GlobalIdx),
}

impl ExportDesc {
    /// returns the external type of `self` according to typing relation,
    /// taking `validation_info` as validation context C
    ///
    /// Note: This method may panic if `self` does not come from the given [`ValidationInfo`].
    /// <https://webassembly.github.io/spec/core/valid/modules.html#exports>
    pub fn extern_type(&self, validation_info: &ValidationInfo) -> ExternType {
        // TODO clean up logic for checking if an exported definition is an
        // import
        match self {
            ExportDesc::FuncIdx(func_idx) => {
                let type_idx = match func_idx
                    .checked_sub(validation_info.imports_length.imported_functions)
                {
                    Some(local_func_idx) => *validation_info.functions.get(local_func_idx).unwrap(),
                    None => validation_info
                        .imports
                        .iter()
                        .filter_map(|import| match import.desc {
                            ImportDesc::Func(type_idx) => Some(type_idx),
                            _ => None,
                        })
                        .nth(*func_idx)
                        .unwrap(),
                };
                let func_type = validation_info
                    .types
                    .get(type_idx)
                    .expect("type indices to always be valid if the validation info is correct");
                // TODO ugly clone that should disappear when types are directly parsed from bytecode instead of vector copies
                ExternType::Func(func_type.clone())
            }
            ExportDesc::TableIdx(table_idx) => {
                let table_type = match table_idx
                    .checked_sub(validation_info.imports_length.imported_tables)
                {
                    Some(local_table_idx) => *validation_info.tables.get(local_table_idx).unwrap(),
                    None => validation_info
                        .imports
                        .iter()
                        .filter_map(|import| match import.desc {
                            ImportDesc::Table(table_type) => Some(table_type),
                            _ => None,
                        })
                        .nth(*table_idx)
                        .unwrap(),
                };
                ExternType::Table(table_type)
            }
            ExportDesc::MemIdx(mem_idx) => {
                let mem_type = match mem_idx
                    .checked_sub(validation_info.imports_length.imported_memories)
                {
                    Some(local_mem_idx) => *validation_info.memories.get(local_mem_idx).unwrap(),
                    None => validation_info
                        .imports
                        .iter()
                        .filter_map(|import| match import.desc {
                            ImportDesc::Mem(mem_type) => Some(mem_type),
                            _ => None,
                        })
                        .nth(*mem_idx)
                        .unwrap(),
                };
                ExternType::Mem(mem_type)
            }
            ExportDesc::GlobalIdx(global_idx) => {
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

    pub fn get_function_idx(&self) -> Option<FuncIdx> {
        match self {
            ExportDesc::FuncIdx(func_idx) => Some(*func_idx),
            _ => None,
        }
    }

    pub fn get_global_idx(&self) -> Option<GlobalIdx> {
        match self {
            ExportDesc::GlobalIdx(global_idx) => Some(*global_idx),
            _ => None,
        }
    }

    pub fn get_memory_idx(&self) -> Option<MemIdx> {
        match self {
            ExportDesc::MemIdx(mem_idx) => Some(*mem_idx),
            _ => None,
        }
    }

    pub fn get_table_idx(&self) -> Option<TableIdx> {
        match self {
            ExportDesc::TableIdx(table_idx) => Some(*table_idx),
            _ => None,
        }
    }
}

impl ExportDesc {
    pub fn read(wasm: &mut WasmReader) -> Result<Self, ValidationError> {
        let desc_id = wasm.read_u8()?;
        let desc_idx = wasm.read_var_u32()?.into_usize();

        let desc = match desc_id {
            0x00 => ExportDesc::FuncIdx(desc_idx),
            0x01 => ExportDesc::TableIdx(desc_idx),
            0x02 => ExportDesc::MemIdx(desc_idx),
            0x03 => ExportDesc::GlobalIdx(desc_idx),
            other => return Err(ValidationError::MalformedExportDescDiscriminator(other)),
        };
        Ok(desc)
    }
}
