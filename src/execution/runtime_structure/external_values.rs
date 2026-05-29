use crate::{
    core::structure::types::ExternTypeRef, execution::runtime_structure::memory_instances::MemInst,
    Config, FuncAddr, GlobalAddr, MemAddr, Store, TableAddr,
};

///<https://webassembly.github.io/spec/core/exec/runtime.html#external-values>
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ExternVal {
    Func(FuncAddr),
    Table(TableAddr),
    Mem(MemAddr),
    Global(GlobalAddr),
}

impl ExternVal {
    /// returns the external type of `self` according to typing relation,
    /// taking `store` as context S.
    ///
    /// # Safety
    /// The caller has to guarantee that `self` came from the same [`Store`] which
    /// is passed now as a reference.
    // TODO make this fn unsafe
    pub(crate) fn extern_type<'store, T: Config>(
        &self,
        store: &'store Store<T>,
    ) -> ExternTypeRef<'store> {
        match self {
            ExternVal::Func(func_addr) => {
                // SAFETY: The caller ensures that self including the function
                // address in self is valid in the given store.
                let function = unsafe { store.inner.functions.get(*func_addr) };
                ExternTypeRef::Func(function.ty())
            }
            ExternVal::Table(table_addr) => {
                // SAFETY: The caller ensures that self including the table
                // address in self is valid in the given store.
                let table = unsafe { store.inner.tables.get(*table_addr) };
                ExternTypeRef::Table(table.ty)
            }
            ExternVal::Mem(mem_addr) => {
                // SAFETY: The caller ensures that self including the memory
                // address in self is valid in the given store.
                let memory = unsafe { store.inner.memories.get(*mem_addr) };
                let ty = match memory {
                    MemInst::Shared(shared_mem_inst) => shared_mem_inst.ty,
                    MemInst::Unshared(unshared_mem_inst) => unshared_mem_inst.ty,
                };
                ExternTypeRef::Mem(ty)
            }
            ExternVal::Global(global_addr) => {
                // SAFETY: The caller ensures that self including the global
                // address in self is valid in the given store.
                let global = unsafe { store.inner.globals.get(*global_addr) };
                ExternTypeRef::Global(global.ty)
            }
        }
    }
}

impl ExternVal {
    pub fn as_func(self) -> Option<FuncAddr> {
        match self {
            ExternVal::Func(func_addr) => Some(func_addr),
            _ => None,
        }
    }

    pub fn as_table(self) -> Option<TableAddr> {
        match self {
            ExternVal::Table(table_addr) => Some(table_addr),
            _ => None,
        }
    }

    pub fn as_mem(self) -> Option<MemAddr> {
        match self {
            ExternVal::Mem(mem_addr) => Some(mem_addr),
            _ => None,
        }
    }

    pub fn as_global(self) -> Option<GlobalAddr> {
        match self {
            ExternVal::Global(global_addr) => Some(global_addr),
            _ => None,
        }
    }
}

/// common convention functions defined for lists of ExternVals, ExternTypes, Exports
/// <https://webassembly.github.io/spec/core/exec/runtime.html#conventions>
/// <https://webassembly.github.io/spec/core/syntax/types.html#id3>
/// <https://webassembly.github.io/spec/core/syntax/modules.html?highlight=convention#id1>
// TODO implement this trait for ExternType lists Export lists
pub trait ExternFilterable {
    fn funcs(self) -> impl Iterator<Item = FuncAddr>;
    fn tables(self) -> impl Iterator<Item = TableAddr>;
    fn mems(self) -> impl Iterator<Item = MemAddr>;
    fn globals(self) -> impl Iterator<Item = GlobalAddr>;
}

impl<'a, I> ExternFilterable for I
where
    I: Iterator<Item = &'a ExternVal>,
{
    fn funcs(self) -> impl Iterator<Item = FuncAddr> {
        self.filter_map(|extern_val| extern_val.as_func())
    }

    fn tables(self) -> impl Iterator<Item = TableAddr> {
        self.filter_map(|extern_val| extern_val.as_table())
    }

    fn mems(self) -> impl Iterator<Item = MemAddr> {
        self.filter_map(|extern_val| extern_val.as_mem())
    }

    fn globals(self) -> impl Iterator<Item = GlobalAddr> {
        self.filter_map(|extern_val| extern_val.as_global())
    }
}
