use crate::core::reader::types::ValType;
use alloc::vec::Vec;
use crate::core::reader::span::Span;

struct FunctionHeader {
    locals: Vec<ValType>,
    expr_span: Span,
}