use crate::{ExternVal, core::decoding::decoder::span::Span};

#[derive(Copy, Clone, Debug)]
pub struct ExportInst {
    pub name: Span,
    pub value: ExternVal,
}
