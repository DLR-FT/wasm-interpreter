use crate::core::structure::modules::indices::{FuncIdx, GlobalIdx, MemIdx, TableIdx};

#[derive(Debug, Clone)]
pub struct Export<'wasm> {
    pub name: &'wasm str,
    pub desc: ExportDesc,
}

#[derive(Debug, Clone)]
pub enum ExportDesc {
    Func(FuncIdx),
    Table(TableIdx),
    Mem(MemIdx),
    Global(GlobalIdx),
}
