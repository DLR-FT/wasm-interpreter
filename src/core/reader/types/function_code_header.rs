use crate::core::reader::span::Span;
use crate::core::reader::types::ValType;
use alloc::vec::Vec;

struct FunctionHeader {
    locals: Vec<ValType>,
    expr_span: Span,
}
