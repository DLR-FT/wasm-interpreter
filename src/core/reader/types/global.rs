use crate::core::reader::span::Span;
use crate::core::structure::types::GlobalType;

#[derive(Debug, Copy, Clone)]
pub struct Global {
    pub ty: GlobalType,
    // TODO validate init_expr during validation and execute during instantiation
    pub init_expr: Span,
}