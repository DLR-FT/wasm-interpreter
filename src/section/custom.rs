use crate::section::{SectionHeader, SectionTy};
use crate::wasm::Wasm;
use crate::Result;
use alloc::string::String;

impl<'a> Wasm<'a> {
    pub fn read_custom_section(&mut self, section_header: SectionHeader) -> Result<()> {
        assert_eq!(section_header.ty, SectionTy::Custom);

        let (name, name_len) =
            self.measure_num_read_bytes(|wasm| wasm.read_name().map(String::from))?;

        // TODO check if known custom section type. For now ignore all custom sections as they cannot invalidate a otherwise valid program.

        // Calculate length of this section's contents
        let contents_len = section_header.contents.len() - name_len;
        trace!("Skipping custom section {:?}", name);
        self.skip_n_bytes(contents_len)
    }
    fn skip_n_bytes(&mut self, n: usize) -> Result<()> {
        for _ in 0..n {
            self.read_u8()?;
        }
        Ok(())
    }
}
