use crate::wasm::span::Span;
use crate::wasm::Wasm;
use crate::Error;
use crate::Result;

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

#[derive(Copy, Clone, Debug)]
pub(crate) struct Section {
    pub ty: SectionTy,
    pub contents: Span,
}

impl<'a> Wasm<'a> {
    pub fn read_section(&mut self) -> Result<Section> {
        let ty: SectionTy = self.strip_u8()?.try_into()?;
        let size: u32 = self.strip_var_u32()?;
        let contents_span = self.make_span(size as usize);

        Ok(Section {
            ty,
            contents: contents_span,
        })
    }
}

pub(crate) struct SectionTypeOrderValidator {
    last: Option<SectionTy>,
}

impl SectionTypeOrderValidator {
    pub fn new() -> Self {
        Self { last: None }
    }
    pub fn validate(&mut self, ty: SectionTy) -> Result<()> {
        if ty == SectionTy::Custom {
            return Ok(());
        }

        let Some(last) = self.last else {
            self.last = Some(ty);
            return Ok(());
        };

        let order_is_valid = Self::SECTION_TYPE_ORDER
            .iter()
            .skip_while(|&&section_ty| section_ty != last)
            .skip(1)
            .any(|&section_ty| section_ty == ty);

        if order_is_valid {
            self.last = Some(ty);
            Ok(())
        } else {
            Err(Error::SectionOutOfOrder(ty))
        }
    }
    const SECTION_TYPE_ORDER: &'static [SectionTy] = &[
        SectionTy::Type,
        SectionTy::Import,
        SectionTy::Function,
        SectionTy::Table,
        SectionTy::Memory,
        SectionTy::Global,
        SectionTy::Export,
        SectionTy::Start,
        SectionTy::Element,
        SectionTy::DataCount,
        SectionTy::Code,
        SectionTy::Data,
    ];
}
