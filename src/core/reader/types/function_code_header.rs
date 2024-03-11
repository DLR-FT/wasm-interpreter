use alloc::vec::Vec;

use crate::core::reader::span::Span;
use crate::core::reader::types::ValType;

struct FunctionHeader {
    locals: Vec<ValType>,
    expr_span: Span,
}
