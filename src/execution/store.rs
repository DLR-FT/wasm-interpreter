use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::iter;

use crate::core::indices::TypeIdx;
use crate::core::reader::span::Span;
use crate::core::reader::types::export::Export;
// use crate::core::reader::types::global::Global;
use crate::core::reader::types::{MemType, TableType, ValType};
use crate::core::sidetable::Sidetable;
// use crate::execution::value::{Ref, Value};
use crate::execution::value::Ref;
use crate::RefType;

use super::global_store::TableInst;

/// The store represents all global state that can be manipulated by WebAssembly programs. It
/// consists of the runtime representation of all instances of functions, tables, memories, and
/// globals, element segments, and data segments that have been allocated during the life time of
/// the abstract machine.
/// <https://webassembly.github.io/spec/core/exec/runtime.html#store>
pub struct Store {
    pub funcs: Vec<FuncInst>,
    pub mems: Vec<MemInst>,
    pub globals: Vec<usize>,
    pub data: Vec<DataInst>,
    pub tables: Vec<usize>,
    pub elements: Vec<ElemInst>,
    pub passive_elem_indexes: Vec<usize>,
    pub exports: Vec<Export>,
}

#[derive(Debug)]
pub enum FuncInst {
    Local(LocalFuncInst),
    Imported(ImportedFuncInst),
}

#[derive(Debug)]
pub struct LocalFuncInst {
    pub ty: TypeIdx,
    pub locals: Vec<ValType>,
    pub code_expr: Span,
    pub sidetable: Sidetable,
}

#[derive(Debug)]
pub struct ImportedFuncInst {
    pub ty: TypeIdx,
    pub module_name: String,
    pub function_name: String,
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

#[derive(Clone, Debug)]
/// <https://webassembly.github.io/spec/core/exec/runtime.html#element-instances>
pub struct ElemInst {
    pub ty: RefType,
    pub references: Vec<Ref>,
}

impl ElemInst {
    pub fn len(&self) -> usize {
        self.references.len()
    }
    pub fn is_empty(&self) -> bool {
        self.references.is_empty()
    }
}

pub struct MemInst {
    #[allow(warnings)]
    pub ty: MemType,
    pub data: Vec<u8>,
}

impl MemInst {
    pub fn new(ty: MemType) -> Self {
        let initial_size = (crate::Limits::MEM_PAGE_SIZE as usize) * ty.limits.min as usize;

        Self {
            ty,
            data: vec![0u8; initial_size],
        }
    }

    pub fn grow(&mut self, delta_pages: usize) {
        self.data
            .extend(iter::repeat(0).take(delta_pages * (crate::Limits::MEM_PAGE_SIZE as usize)))
    }

    /// Can never be bigger than 65,356 pages
    pub fn size(&self) -> usize {
        self.data.len() / (crate::Limits::MEM_PAGE_SIZE as usize)
    }
}

// pub struct GlobalInst {
//     pub global: Global,
//     /// Must be of the same type as specified in `ty`
//     pub value: Value,
// }

pub struct DataInst {
    pub data: Vec<u8>,
}
