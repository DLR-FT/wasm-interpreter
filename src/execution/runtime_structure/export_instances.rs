use crate::{core::decoding::decoder::span::Span, ExternVal};

#[derive(Copy, Clone, Debug)]
pub struct ExportInst {
    pub name: Span,
    pub value: ExternVal,
}
