use alloc::vec::Vec;
use core::fmt;

use crate::{
    core::{
        decoding::decoder::span::Span,
        structure::modules::indices::{FuncIdx, TableIdx},
    },
    RefType,
};

#[derive(Clone)]
pub struct ElemType {
    pub init: ElemItems,
    pub mode: ElemMode,
}

impl fmt::Debug for ElemType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ElemType {{\n\tinit: {:?},\n\tmode: {:?},\n\t#ty: {:?}\n}}",
            self.init,
            self.mode,
            self.init.ty()
        )
    }
}

impl ElemType {
    pub fn ty(&self) -> RefType {
        self.init.ty()
    }

    pub fn to_ref_type(&self) -> RefType {
        match self.init {
            ElemItems::Exprs(rref, _) => rref,
            ElemItems::RefFuncs(_) => RefType::FuncRef,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ElemItems {
    RefFuncs(Vec<FuncIdx>),
    Exprs(RefType, Vec<Span>),
}

impl ElemItems {
    pub fn ty(&self) -> RefType {
        match self {
            Self::RefFuncs(_) => RefType::FuncRef,
            // the mapping for shortened lists above is always true, as the binary format
            // either parses an elemkind or assumes funcref, and the current spec always maps a well-formed elemkind to a funcref
            // https://webassembly.github.io/spec/core/binary/modules.html#element-section
            Self::Exprs(rty, _) => *rty,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::RefFuncs(ref_funcs) => ref_funcs.len(),
            Self::Exprs(_, exprs) => exprs.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ElemMode {
    Passive,
    Active(ActiveElem),
    Declarative,
}

#[derive(Debug, Clone)]
pub struct ActiveElem {
    pub table_idx: TableIdx,
    pub init_expr: Span,
}
