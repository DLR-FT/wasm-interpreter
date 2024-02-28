use crate::wasm::span::Span;
use crate::wasm::Wasm;
use crate::Error;
use crate::Result;

pub mod code;
pub mod custom;
pub mod export;
pub mod function;
pub mod r#type;

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
    type Error = Error;

    fn try_from(ty: u8) -> core::result::Result<Self, Self::Error> {
        use SectionTy::*;
        let ty = match ty {
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
            other => return Err(Error::InvalidSectionType(other)),
        };

        Ok(ty)
    }
}

#[derive(Debug)]
pub(crate) struct SectionHeader {
    pub ty: SectionTy,
    pub contents: Span,
}

impl<'a> Wasm<'a> {
    pub fn read_section_header(&mut self) -> Result<SectionHeader> {
        let ty: SectionTy = self.read_u8()?.try_into()?;
        let size: u32 = self.read_var_u32()?;
        let contents_span = self.make_span(size as usize);

        Ok(SectionHeader {
            ty,
            contents: contents_span,
        })
    }
}
