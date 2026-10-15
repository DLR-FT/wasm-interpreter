use crate::{GlobalType, core::decoding::decoder::span::Span};

#[derive(Debug, Copy, Clone)]
pub struct Global {
    pub ty: GlobalType,
    pub init_expr: Span,
}
