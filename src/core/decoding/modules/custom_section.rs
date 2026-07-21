use crate::{
    core::decoding::decoder::{span::Span, WasmDecoder},
    DecodingError, ValidationError,
};

#[derive(Debug, Clone)]
pub struct CustomSection<'wasm> {
    pub name: &'wasm str,
    pub contents: &'wasm [u8],
}

impl<'wasm> CustomSection<'wasm> {
    // TODO this should return a Result<_, DecodingError>
    pub(crate) fn decode(
        wasm: &mut WasmDecoder<'wasm>,
        section_contents: Span,
    ) -> Result<CustomSection<'wasm>, ValidationError> {
        // customsec ::= section_0(custom)
        // custom ::= name byte*
        // name ::= b*:vec(byte) => name (if utf8(name) = b*)
        // vec(B) ::= n:u32 (x:B)^n => x^n
        let name = wasm.decode_name()?;

        let section_start = wasm.pc;
        let section_end = section_contents
            .from()
            .checked_add(section_contents.len())
            .ok_or(DecodingError::SectionSizeMismatch)?;

        let contents = wasm
            .full_wasm_binary
            .get(section_start..section_end)
            .ok_or(DecodingError::SectionSizeMismatch)?;

        let section_len = section_end
            .checked_sub(section_start)
            .expect("section start <= section end always");

        wasm.skip(section_len)?;

        Ok(CustomSection { name, contents })
    }
}
