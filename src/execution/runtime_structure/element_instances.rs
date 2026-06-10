use alloc::vec::Vec;

use crate::{Ref, RefType};

#[derive(Clone, Debug)]
/// <https://webassembly.github.io/spec/core/exec/runtime.html#element-instances>
pub struct ElemInst {
    pub _ty: RefType,
    pub references: Vec<Ref>,
}

impl ElemInst {
    pub fn len(&self) -> usize {
        self.references.len()
    }
}
