use crate::{core::structure::modules::indices::TypeIdx, GlobalType, MemType, TableType};

#[derive(Debug, Clone)]
pub struct Import<'wasm> {
    pub module_name: &'wasm str,
    pub name: &'wasm str,
    pub desc: ImportDesc,
}

#[derive(Debug, Clone)]
pub enum ImportDesc {
    Func(TypeIdx),
    Table(TableType),
    Mem(MemType),
    Global(GlobalType),
}
