use crate::{core::reader::span::Span, GlobalType};

#[derive(Debug, Copy, Clone)]
pub struct Global {
    pub ty: GlobalType,
    pub init_expr: Span,
}
