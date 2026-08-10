use crate::{
    core::{decoding::decoder::span::Span, structure::modules::indices::TypeIdx},
    GlobalType, MemType, TableType,
};

#[derive(Debug, Clone)]
pub struct Import {
    pub module_name: Span,
    pub name: Span,
    pub desc: ImportDesc,
}

#[derive(Debug, Clone)]
pub enum ImportDesc {
    Func(TypeIdx),
    Table(TableType),
    Mem(MemType),
    Global(GlobalType),
}
