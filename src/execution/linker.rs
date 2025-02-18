use core::fmt::Error;

use alloc::vec::Vec;

use crate::{
    core::reader::types::FuncType, DataInst, ElemInst, FuncInst, GlobalInst, MemInst, TableInst,
};

#[derive(Debug)]
pub enum LinkerError {
    Custom(&'static str),
}

#[derive(Debug, Default)]
struct LinkerFunc {
    pub exported: bool,
}

// #[derive(Debug)]
// enum LinkerMem {}

// #[derive(Debug)]
// enum LinkerGlobal {}

// #[derive(Debug)]
// enum LinkerTable {}

// #[derive(Debug)]
// enum LinkerData {}

// #[derive(Debug)]
// enum LinkerElem {}

#[derive(Debug, Default)]
pub struct Linker {
    pub functions: Vec<LinkerFunc>,
    // pub memories: Vec<LinkerMem>,
    // pub globals: Vec<LinkerGlobal>,
    // pub tables: Vec<LinkerTable>,
    // pub data: Vec<LinkerData>,
    // pub elements: Vec<LinkerElem>,

    // for the future when we'll need to allow shadowing like wasmtime and wasmi do
    #[allow(dead_code)]
    allow_shadowing: bool,
}

pub struct LinkerObject {
    pub exported: bool,
    pub inner: LinkerObjectInner,
}

#[derive(Debug)]
pub enum LinkerObjectInner {
    Func(LinkerFunc),
    // Mem(LinkerMem),
    // Global(LinkerGlobal),
    // Table(LinkerTable),
    // Data(LinkerData),
    // Elem(LinkerElem),
}

impl Into<LinkerObjectInner> for LinkerFunc {
    fn into(self) -> LinkerObjectInner {
        LinkerObjectInner::Func(self)
    }
}
// impl Into<LinkerObjectInner> for LinkerMem {
//     fn into(self) -> LinkerObjectInner {
//         LinkerObjectInner::Mem(self)
//     }
// }
// impl Into<LinkerObjectInner> for LinkerGlobal {
//     fn into(self) -> LinkerObjectInner {
//         LinkerObjectInner::Global(self)
//     }
// }
// impl Into<LinkerObjectInner> for LinkerTable {
//     fn into(self) -> LinkerObjectInner {
//         LinkerObjectInner::Table(self)
//     }
// }
// impl Into<LinkerObjectInner> for LinkerData {
//     fn into(self) -> LinkerObjectInner {
//         LinkerObjectInner::Data(self)
//     }
// }
// impl Into<LinkerObjectInner> for LinkerElem {
//     fn into(self) -> LinkerObjectInner {
//         LinkerObjectInner::Elem(self)
//     }
// }

pub enum LinkerObjectType {
    Func,
    // Mem,
    // Global,
    // Table,
    // Data,
    // Elem,
}

impl Linker {
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(dead_code)]
    pub fn allow_shadowing(&mut self, allow: bool) {
        self.allow_shadowing = allow;
    }

    /// Index should be set to Some(usize) only when we wish to replace a definition
    pub fn insert(
        &mut self,
        item: LinkerObjectInner,
        index: Option<usize>,
    ) -> Result<(usize, LinkerObjectType), LinkerError> {
        match item {
            LinkerObjectInner::Func(func) => {
                if let Some(index) = index {
                    if !self.allow_shadowing {
                        return Err(LinkerError::Custom("Duplicate definition"));
                    }
                    if self.functions.len() >= index {
                        return Err(LinkerError::Custom("Out of bounds"));
                    }

                    // todo: do some more type checking on the function type

                    self.functions[index] = func;
                    Ok((index, LinkerObjectType::Func))
                } else {
                    self.functions.push(func);
                    Ok((self.functions.len() - 1, LinkerObjectType::Func))
                }
            } // LinkerObjectInner::Mem(_mem) => todo!(),
              // LinkerObjectInner::Global(_global) => todo!(),
              // LinkerObjectInner::Table(_table) => todo!(),
              // LinkerObjectInner::Data(_data) => todo!(),
              // LinkerObjectInner::Elem(_elem) => todo!(),
        }
    }

    pub fn set_export_property(
        &mut self,
        ty: LinkerObjectType,
        index: usize,
        export: bool,
    ) -> Result<(), LinkerError> {
        match ty {
            LinkerObjectType::Func => {
                if self.functions.len() >= index {
                    return Err(LinkerError::Custom("Out of bounds"));
                } else {
                    self.functions[index].exported = export;
                    Ok(())
                }
            }
        }
    }
}
