use alloc::vec::Vec;

use crate::{
    core::{decoding::reader::span::Span, structure::modules::indices::TypeIdx},
    FuncType, Hostcode, ModuleAddr, ValType,
};

#[derive(Debug)]
// TODO does not match the spec FuncInst
pub enum FuncInst {
    WasmFunc(WasmFuncInst),
    HostFunc(HostFuncInst),
}

#[derive(Debug)]
pub struct WasmFuncInst {
    pub function_type: FuncType,
    pub _ty: TypeIdx,
    pub locals: Vec<ValType>,
    pub code_expr: Span,
    ///index of the sidetable corresponding to the beginning of this functions code
    pub stp: usize,

    // implicit back ref required for function invocation and is in the spec
    // TODO module_addr or module ref?
    pub module_addr: ModuleAddr,
}

#[derive(Debug)]
pub struct HostFuncInst {
    pub function_type: FuncType,
    pub hostcode: Hostcode,
}

impl FuncInst {
    pub fn ty(&self) -> &FuncType {
        match self {
            FuncInst::WasmFunc(wasm_func_inst) => &wasm_func_inst.function_type,
            FuncInst::HostFunc(host_func_inst) => &host_func_inst.function_type,
        }
    }
}
