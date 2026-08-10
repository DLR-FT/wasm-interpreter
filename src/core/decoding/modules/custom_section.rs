use crate::{
    core::decoding::decoder::{span::Span, WasmDecoder},
    DecodingError,
};

#[derive(Debug, Clone)]
pub struct CustomSection {
    pub name: Span,
    pub contents: Span,
}

impl CustomSection {
    pub(crate) fn decode(
        wasm: &mut WasmDecoder,
        section_contents: Span,
    ) -> Result<CustomSection, DecodingError> {
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

        let section_len = section_end
            .checked_sub(section_start)
            .ok_or(DecodingError::SectionSizeMismatch)?;

        let contents = Span {
            from: section_start,
            len: section_len,
        };

        wasm.skip(section_len)?;

        Ok(CustomSection { name, contents })
    }
}
