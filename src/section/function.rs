use alloc::vec::Vec;

use crate::section::{SectionHeader, SectionTy};
use crate::wasm::indices::TypeIdx;
use crate::wasm::Wasm;
use crate::Result;

impl<'a> Wasm<'a> {
    pub fn read_function_section(&mut self, section_header: SectionHeader) -> Result<Vec<TypeIdx>> {
        assert_eq!(section_header.ty, SectionTy::Function);

        let typeidxs = self.read_vec(|wasm| wasm.read_var_u32().map(|idx| TypeIdx(idx)))?;
        debug!("Function section read: {:?}", typeidxs);
        Ok(typeidxs)
    }
}
