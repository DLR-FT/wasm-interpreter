use alloc::borrow::ToOwned;
use alloc::string::String;

use crate::core::indices::{FuncIdx, GlobalIdx, MemIdx, TableIdx};
use crate::core::reader::{WasmReadable, WasmReader};
use crate::execution::assert_validated::UnwrapValidatedExt;
use crate::{unreachable_validated, Error, Result};

#[derive(Debug)]
pub struct Export {
    pub name: String,
    pub desc: ExportDesc,
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

#[derive(Debug)]
pub enum ExportDesc {
    FuncIdx(FuncIdx),
    TableIdx(TableIdx),
    MemIdx(MemIdx),
    GlobalIdx(GlobalIdx),
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
            other => unreachable_validated!(),
        }
    }
}
