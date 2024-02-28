use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::vec::Vec;

use crate::section::{SectionHeader, SectionTy};
use crate::wasm::indices::{FuncIdx, GlobalIdx, MemIdx, TableIdx};
use crate::wasm::Wasm;
use crate::{Error, Result};

#[derive(Debug)]
pub struct Export {
    pub name: String,
    pub desc: ExportDesc,
}

#[derive(Debug)]
pub enum ExportDesc {
    FuncIdx(FuncIdx),
    TableIdx(TableIdx),
    MemIdx(MemIdx),
    GlobalIdx(GlobalIdx),
}

impl<'a> Wasm<'a> {
    pub fn read_export_section(&mut self, section_header: SectionHeader) -> Result<Vec<Export>> {
        assert_eq!(section_header.ty, SectionTy::Export);

        let exports = self.read_vec(|wasm| wasm.read_export())?;
        debug!("Export section read: {:?}", &exports);
        Ok(exports)
    }

    fn read_export(&mut self) -> Result<Export> {
        let name = self.read_name()?.to_owned();

        let desc_id = self.read_u8()?;
        let desc_content = self.read_var_u32()?;

        let desc = match desc_id {
            0x00 => ExportDesc::FuncIdx(FuncIdx(desc_content)),
            0x01 => ExportDesc::TableIdx(TableIdx(desc_content)),
            0x02 => ExportDesc::MemIdx(MemIdx(desc_content)),
            0x03 => ExportDesc::GlobalIdx(GlobalIdx(desc_content)),
            other => return Err(Error::InvalidExportDesc(other)),
        };

        Ok(Export { name, desc })
    }
}
