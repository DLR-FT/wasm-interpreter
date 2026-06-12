use crate::ExternVal;

#[derive(Copy, Clone, Debug)]
pub struct ExportInst<'wasm> {
    pub name: &'wasm str,
    pub value: ExternVal,
}
