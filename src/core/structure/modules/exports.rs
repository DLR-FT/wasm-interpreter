use crate::core::{
    decoding::decoder::span::Span,
    structure::modules::indices::{FuncIdx, GlobalIdx, MemIdx, TableIdx},
};

#[derive(Debug, Clone)]
pub struct Export {
    pub name: Span,
    pub desc: ExportDesc,
}

#[derive(Debug, Clone)]
pub enum ExportDesc {
    Func(FuncIdx),
    Table(TableIdx),
    Mem(MemIdx),
    Global(GlobalIdx),
}
