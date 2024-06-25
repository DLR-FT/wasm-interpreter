use alloc::vec;
use alloc::vec::Vec;
use core::iter;

use crate::core::indices::TypeIdx;
use crate::core::reader::span::Span;
use crate::core::reader::types::global::Global;
use crate::core::reader::types::{MemType, TableType, ValType};
use crate::execution::value::{Ref, Value};

/// <https://webassembly.github.io/spec/core/exec/runtime.html#store>
pub struct Store {
    pub funcs: Vec<FuncInst>,
    // tables: Vec<TableInst>,
    pub mems: Vec<MemInst>,
    pub globals: Vec<GlobalInst>,
}

pub struct FuncInst {
    pub ty: TypeIdx,
    pub locals: Vec<ValType>,
    pub code_expr: Span,
}

#[allow(dead_code)]
pub struct TableInst {
    pub ty: TableType,
    pub elem: Vec<Ref>,
}

pub struct MemInst {
    #[allow(warnings)]
    pub ty: MemType,
    pub data: Vec<u8>,
}

impl MemInst {
    const PAGE_SIZE: usize = 1 << 16;
    pub fn new(ty: MemType) -> Self {
        let initial_size = Self::PAGE_SIZE * ty.limits.min as usize;

        Self {
            ty,
            data: vec![0u8; initial_size],
        }
    }

    #[allow(dead_code)]
    pub fn grow(&mut self, delta_pages: usize) {
        self.data
            .extend(iter::repeat(0).take(delta_pages * Self::PAGE_SIZE))
    }

    #[allow(dead_code)]
    pub fn size(&self) -> usize {
        self.data.len() / Self::PAGE_SIZE
    }
}

pub struct GlobalInst {
    pub global: Global,
    /// Must be of the same type as specified in `ty`
    pub value: Value,
}
