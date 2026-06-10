use alloc::{collections::btree_map::BTreeMap, string::String};

use crate::{
    core::{
        sidetable::Sidetable,
        structure::modules::indices::{
            DataIdx, ElemIdx, FuncIdx, GlobalIdx, IdxVec, MemIdx, TableIdx, TypeIdx,
        },
    },
    DataAddr, ElemAddr, ExternVal, FuncAddr, FuncType, GlobalAddr, MemAddr, TableAddr,
};

/// <https://webassembly.github.io/spec/core/exec/runtime.html#module-instances>/
///
/// # Safety
///
/// All indices contained in a module instance must be valid in their associated
/// index vectors from the same module instance.
#[derive(Debug)]
pub struct ModuleInst<'b> {
    pub types: IdxVec<TypeIdx, FuncType>,
    pub func_addrs: IdxVec<FuncIdx, FuncAddr>,
    pub table_addrs: IdxVec<TableIdx, TableAddr>,
    pub mem_addrs: IdxVec<MemIdx, MemAddr>,
    pub global_addrs: IdxVec<GlobalIdx, GlobalAddr>,
    pub elem_addrs: IdxVec<ElemIdx, ElemAddr>,
    pub data_addrs: IdxVec<DataIdx, DataAddr>,
    ///<https://webassembly.github.io/spec/core/exec/runtime.html#export-instances>
    /// matches the list of ExportInst structs in the spec, however the spec never uses the name attribute
    /// except during linking, which is up to the embedder to implement.
    /// therefore this is a map data structure instead.
    pub exports: BTreeMap<String, ExternVal>,

    // TODO the bytecode is not in the spec, but required for re-parsing
    pub wasm_bytecode: &'b [u8],

    // sidetable is not in the spec, but required for control flow
    pub sidetable: Sidetable,
}
