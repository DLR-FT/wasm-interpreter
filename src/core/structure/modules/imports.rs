use crate::{
    core::{decoding::decoder::span::Span, structure::modules::indices::TypeIdx},
    GlobalType, MemType, TableType,
};

#[derive(Debug, Clone)]
pub struct Import {
    pub(crate) module_name: Span,
    pub(crate) name: Span,
    pub(crate) desc: ImportDesc,
}

#[derive(Debug, Clone)]
pub enum ImportDesc {
    Func(TypeIdx),
    Table(TableType),
    Mem(MemType),
    Global(GlobalType),
}
