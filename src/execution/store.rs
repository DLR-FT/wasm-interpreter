use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::iter;

use crate::core::indices::TypeIdx;
use crate::core::reader::span::Span;
use crate::core::reader::types::export::Export;
use crate::core::reader::types::global::Global;
use crate::core::reader::types::{MemType, TableType, ValType};
use crate::execution::value::{Ref, Value};

/// The store represents all global state that can be manipulated by WebAssembly programs. It
/// consists of the runtime representation of all instances of functions, tables, memories, and
/// globals, element segments, and data segments that have been allocated during the life time of
/// the abstract machine.
/// <https://webassembly.github.io/spec/core/exec/runtime.html#store>
#[derive(Debug)]
pub struct Store {
    pub funcs: Vec<FuncInst>,
    // tables: Vec<TableInst>,
    pub mems: Vec<MemInst>,
    pub globals: Vec<GlobalInst>,
    pub exports: Vec<Export>,
}

#[derive(Debug)]
pub enum FuncInst {
    Local(LocalFuncInst),
    Imported(ImportedFuncInst),
}

impl FuncInst {
    pub fn ty(&self) -> TypeIdx {
        match self {
            FuncInst::Local(f) => f.ty,
            FuncInst::Imported(f) => f.ty,
        }
    }

    pub fn try_into_local(&self) -> Option<&LocalFuncInst> {
        match self {
            FuncInst::Local(f) => Some(f),
            FuncInst::Imported(_) => None,
        }
    }

    pub fn try_into_imported(&self) -> Option<&ImportedFuncInst> {
        match self {
            FuncInst::Local(_) => None,
            FuncInst::Imported(f) => Some(f),
        }
    }
}

#[derive(Debug)]
pub struct LocalFuncInst {
    pub ty: TypeIdx,
    pub locals: Vec<ValType>,
    pub code_expr: Span,
}

#[derive(Debug)]
pub struct ImportedFuncInst {
    pub ty: TypeIdx,
    #[allow(dead_code)]
    pub module_name: String,
    #[allow(dead_code)]
    pub function_name: String,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct TableInst {
    pub ty: TableType,
    pub elem: Vec<Ref>,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct GlobalInst {
    pub global: Global,
    /// Must be of the same type as specified in `ty`
    pub value: Value,
}
