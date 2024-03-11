use crate::core::indices::TypeIdx;
use alloc::vec::Vec;

use crate::core::reader::span::Span;
use crate::core::reader::types::{FuncType, MemType, TableType, ValType};
use crate::execution::value::Ref;

/// <https://webassembly.github.io/spec/core/exec/runtime.html#store>
pub struct Store {
    pub funcs: Vec<FuncInst>,
    // tables: Vec<TableInst>,
    pub mems: Vec<MemInst>,
}

pub struct FuncInst {
    pub ty: TypeIdx,
    pub locals: Vec<ValType>,
    pub code_expr: Span,
}

pub struct TableInst {
    pub ty: TableType,
    pub elem: Vec<Ref>,
}

pub struct MemInst {
    pub ty: MemType,
    pub data: Vec<u8>,
}
