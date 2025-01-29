use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::core::indices::TypeIdx;
use crate::core::reader::span::Span;
use crate::core::reader::types::export::Export;
use crate::core::reader::types::global::Global;
use crate::core::reader::types::{MemType, TableType, ValType};
use crate::core::sidetable::Sidetable;
use crate::execution::value::{Ref, Value};
use crate::linear_memory::LinearMemory;
use crate::RefType;

/// The store represents all global state that can be manipulated by WebAssembly programs. It
/// consists of the runtime representation of all instances of functions, tables, memories, and
/// globals, element segments, and data segments that have been allocated during the life time of
/// the abstract machine.
/// <https://webassembly.github.io/spec/core/exec/runtime.html#store>
pub struct Store {
    pub funcs: Vec<FuncInst>,
    pub mems: Vec<MemInst>,
    pub globals: Vec<GlobalInst>,
    pub data: Vec<DataInst>,
    pub tables: Vec<TableInst>,
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

#[derive(Debug)]
pub struct TableInst {
    pub ty: TableType,
    pub elem: Vec<Ref>,
}

impl TableInst {
    pub fn len(&self) -> usize {
        self.elem.len()
    }

    pub fn is_empty(&self) -> bool {
        self.elem.is_empty()
    }

    pub fn new(ty: TableType) -> Self {
        Self {
            ty,
            elem: vec![Ref::default_from_ref_type(ty.et); ty.lim.min as usize],
        }
    }
}

pub enum MemInst {
    Local(LocalMemInst),
    Imported(ImportedMemInst),
}

pub struct LocalMemInst {
    pub ty: MemType,
    pub mem: LinearMemory,
}

pub struct ImportedMemInst {
    pub ty: MemType,
    pub module_name: String,
    pub function_name: String,
}

impl MemInst {
    pub fn ty(&self) -> MemType {
        match self {
            MemInst::Local(f) => f.ty,
            MemInst::Imported(f) => f.ty,
        }
    }

    pub fn try_into_local(&mut self) -> Option<&mut LocalMemInst> {
        match self {
            MemInst::Local(f) => Some(f),
            MemInst::Imported(_) => None,
        }
    }

    pub fn try_into_imported(&mut self) -> Option<&mut ImportedMemInst> {
        match self {
            MemInst::Local(_) => None,
            MemInst::Imported(f) => Some(f),
        }
    }
}

impl LocalMemInst {
    pub fn new(ty: MemType) -> Self {
        Self {
            ty,
            mem: LinearMemory::new_with_initial_pages(ty.limits.min.try_into().unwrap()),
        }
    }

    pub fn grow(&mut self, delta_pages: usize) {
        self.mem.grow(delta_pages.try_into().unwrap())
    }

    /// Can never be bigger than 65,356 pages
    pub fn size(&self) -> usize {
        self.mem.len() / (crate::Limits::MEM_PAGE_SIZE as usize)
    }
}

pub struct GlobalInst {
    pub global: Global,
    /// Must be of the same type as specified in `ty`
    pub value: Value,
}

pub struct DataInst {
    pub data: Vec<u8>,
}
