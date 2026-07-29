use crate::{
    core::{
        decoding::decoder::{span::Span, WasmDecoder},
        utils::ToUsizeExt,
    },
    DecodingError,
};

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum SectionTy {
    Custom = 0,
    Type = 1,
    Import = 2,
    Function = 3,
    Table = 4,
    Memory = 5,
    Global = 6,
    Export = 7,
    Start = 8,
    Element = 9,
    Code = 10,
    Data = 11,
    DataCount = 12,
}

impl TryFrom<u8> for SectionTy {
    type Error = DecodingError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use SectionTy::*;
        let ty = match value {
            0 => Custom,
            1 => Type,
            2 => Import,
            3 => Function,
            4 => Table,
            5 => Memory,
            6 => Global,
            7 => Export,
            8 => Start,
            9 => Element,
            10 => Code,
            11 => Data,
            12 => DataCount,
            other => return Err(DecodingError::MalformedSectionTypeDiscriminator(other)),
        };
        Ok(ty)
    }
}

impl SectionTy {
    pub fn decode(wasm: &mut WasmDecoder) -> Result<Self, DecodingError> {
        wasm.decode_u8().and_then(Self::try_from)
    }

    pub fn peek(wasm: &WasmDecoder) -> Result<Self, DecodingError> {
        wasm.peek_u8().and_then(Self::try_from)
    }
}

/// Returns `None` if section types do not match or if the end of the bytecode was reached.
pub fn decode_section_if_ty_matches<'wasm, T, E>(
    wasm: &mut WasmDecoder<'wasm>,
    expected_ty: SectionTy,
    section_consumer: impl FnOnce(&mut WasmDecoder<'wasm>, Span) -> Result<T, E>,
) -> Result<Option<T>, E>
where
    E: From<DecodingError>,
{
    let section_ty = match SectionTy::peek(wasm) {
        Ok(section_ty) => section_ty,
        Err(DecodingError::Eof) => return Ok(None),
        Err(other) => return Err(other.into()),
    };

    if section_ty != expected_ty {
        return Ok(None);
    }
    let _ = SectionTy::decode(wasm).expect("this to be Ok because of the previous peek call");

    let size: u32 = wasm.decode_var_u32()?;
    let contents_span = wasm.make_span(size.into_usize())?;

    let t = section_consumer(wasm, contents_span)?;

    if wasm.pc != contents_span.from() + contents_span.len() {
        return Err(DecodingError::SectionSizeMismatch.into());
    }

    Ok(Some(t))
}
