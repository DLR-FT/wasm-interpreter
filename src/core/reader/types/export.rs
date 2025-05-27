use alloc::borrow::ToOwned;
use alloc::string::String;

use crate::core::indices::{FuncIdx, GlobalIdx, MemIdx, TableIdx};
use crate::core::reader::{WasmReadable, WasmReader};
use crate::execution::assert_validated::UnwrapValidatedExt;
use crate::{unreachable_validated, Error, Result, ValidationInfo};

use super::ExternType;

#[derive(Debug, Clone)]
pub struct Export {
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    pub desc: ExportDesc,
}

impl Export {
    /// returns the external type of `self` according to typing relation,
    /// taking `validation_info` as validation context C
    /// may fail if the external type is not possible to infer with C
    /// <https://webassembly.github.io/spec/core/valid/modules.html#exports>
    #[allow(unused)]
    pub fn extern_type(&self, validation_info: &ValidationInfo) -> Result<ExternType> {
        self.desc.extern_type(validation_info)
    }
}

impl WasmReadable for Export {
    fn read(wasm: &mut WasmReader) -> Result<Self> {
        let name = wasm.read_name()?.to_owned();
        let desc = ExportDesc::read(wasm)?;
        Ok(Export { name, desc })
    }

    fn read_unvalidated(wasm: &mut WasmReader) -> Self {
        let name = wasm.read_name().unwrap_validated().to_owned();
        let desc = ExportDesc::read_unvalidated(wasm);
        Export { name, desc }
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
    /// may fail if the external type is not possible to infer with C
    /// <https://webassembly.github.io/spec/core/valid/modules.html#exports>
    pub fn extern_type(&self, validation_info: &ValidationInfo) -> Result<ExternType> {
        Ok(match self {
            ExportDesc::FuncIdx(func_idx) => {
                let type_idx = validation_info
                    .functions
                    .get(*func_idx)
                    .ok_or(Error::InvalidFuncTypeIdx)?;
                let func_type = validation_info
                    .types
                    .get(*type_idx)
                    .ok_or(Error::InvalidFuncType)?;
                // TODO ugly clone that should disappear when types are directly parsed from bytecode instead of vector copies
                ExternType::Func(func_type.clone())
            }
            // TODO more accurate errors here
            ExportDesc::TableIdx(table_idx) => ExternType::Table(
                *validation_info
                    .tables
                    .get(*table_idx)
                    .ok_or(Error::InvalidLocalIdx)?,
            ),
            ExportDesc::MemIdx(mem_idx) => ExternType::Mem(
                *validation_info
                    .memories
                    .get(*mem_idx)
                    .ok_or(Error::InvalidLocalIdx)?,
            ),
            ExportDesc::GlobalIdx(global_idx) => ExternType::Global(
                validation_info
                    .globals
                    .get(*global_idx)
                    .ok_or(Error::InvalidGlobalIdx(*global_idx))?
                    .ty,
            ),
        })
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

impl WasmReadable for ExportDesc {
    fn read(wasm: &mut WasmReader) -> Result<Self> {
        let desc_id = wasm.read_u8()?;
        let desc_idx = wasm.read_var_u32()? as usize;

        let desc = match desc_id {
            0x00 => ExportDesc::FuncIdx(desc_idx),
            0x01 => ExportDesc::TableIdx(desc_idx),
            0x02 => ExportDesc::MemIdx(desc_idx),
            0x03 => ExportDesc::GlobalIdx(desc_idx),
            other => return Err(Error::InvalidExportDesc(other)),
        };
        Ok(desc)
    }

    fn read_unvalidated(wasm: &mut WasmReader) -> Self {
        let desc_id = wasm.read_u8().unwrap_validated();
        let desc_idx = wasm.read_var_u32().unwrap_validated() as usize;

        match desc_id {
            0x00 => ExportDesc::FuncIdx(desc_idx),
            0x01 => ExportDesc::TableIdx(desc_idx),
            0x02 => ExportDesc::MemIdx(desc_idx),
            0x03 => ExportDesc::GlobalIdx(desc_idx),
            _other => unreachable_validated!(),
        }
    }
}
