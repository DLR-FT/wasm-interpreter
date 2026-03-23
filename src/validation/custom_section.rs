use crate::{
    core::reader::{
        section_header::{SectionHeader, SectionTy},
        WasmReader,
    },
    ValidationError,
};

#[derive(Debug, Clone)]
pub struct CustomSection<'wasm> {
    pub name: &'wasm str,
    pub contents: &'wasm [u8],
}

impl<'wasm> CustomSection<'wasm> {
    pub(crate) fn read_and_validate(
        wasm: &mut WasmReader<'wasm>,
        header: SectionHeader,
    ) -> Result<CustomSection<'wasm>, ValidationError> {
        assert_eq!(header.ty, SectionTy::Custom);

        // customsec ::= section_0(custom)
        // custom ::= name byte*
        // name ::= b*:vec(byte) => name (if utf8(name) = b*)
        // vec(B) ::= n:u32 (x:B)^n => x^n
        let name = wasm.read_name()?;

        let section_start = wasm.pc;
        let section_end = header
            .contents
            .from()
            .checked_add(header.contents.len())
            .ok_or(ValidationError::InvalidCustomSectionLength)?;

        let contents = wasm
            .full_wasm_binary
            .get(section_start..section_end)
            .ok_or(ValidationError::InvalidCustomSectionLength)?;

        let section_len = section_end
            .checked_sub(section_start)
            .expect("section start <= section end always");

        wasm.skip(section_len)?;

        Ok(CustomSection { name, contents })
    }
}
