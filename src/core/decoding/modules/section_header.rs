use crate::{
    core::{
        decoding::reader::{span::Span, WasmDecoder},
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

impl SectionTy {
    pub fn decode(wasm: &mut WasmDecoder) -> Result<Self, DecodingError> {
        use SectionTy::*;
        let ty = match wasm.decode_u8()? {
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

#[derive(Debug)]
pub(crate) struct SectionHeader {
    pub ty: SectionTy,
    pub contents: Span,
}

impl SectionHeader {
    pub fn decode(wasm: &mut WasmDecoder) -> Result<Self, DecodingError> {
        let ty = SectionTy::decode(wasm)?;
        let size: u32 = wasm.decode_var_u32()?;
        let contents_span = wasm.make_span(size.into_usize())?;

        Ok(SectionHeader {
            ty,
            contents: contents_span,
        })
    }
}
