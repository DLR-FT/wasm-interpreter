use alloc::vec::Vec;

use crate::core::reader::span::Span;
use crate::core::reader::types::ValType;

#[allow(dead_code)]
struct FunctionHeader {
    locals: Vec<ValType>,
    expr_span: Span,
}
