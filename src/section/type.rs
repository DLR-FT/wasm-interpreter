use crate::section::{SectionHeader, SectionTy};
use crate::wasm::types::FuncType;
use crate::wasm::Wasm;
use crate::Result;
use alloc::vec::Vec;

impl<'a> Wasm<'a> {
    pub fn read_type_section(&mut self, section_header: SectionHeader) -> Result<Vec<FuncType>> {
        assert_eq!(section_header.ty, SectionTy::Type);

        let functypes = self.read_vec(|wasm| wasm.read_functype())?;
        debug!("Type section read: {:?}", &functypes);
        Ok(functypes)
    }
}
