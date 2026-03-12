use core::{
    num::NonZeroU64,
    ops::{Deref, DerefMut},
};

use alloc::vec::Vec;
use wasm::{
    ExternVal, Hostcode, InstantiationOutcome,
    addrs::{FuncAddr, GlobalAddr, MemAddr, ModuleAddr, TableAddr},
    resumable::{HostCall, HostResumable, HostThing, Resumable, RunState, WasmResumable},
};

use crate::{AbstractStored, StoreId, StoredValue};

/// A generic stored wrapper. This is used to wrap `struct` types such as
/// [`FuncAddr`], [`MemAddr`], etc.
pub struct Stored<T> {
    id: StoreId,
    inner: T,
}

impl<T> AbstractStored for Stored<T> {
    type BareTy = T;

    unsafe fn from_bare(bare_value: Self::BareTy, id: StoreId) -> Self {
        Self {
            inner: bare_value,
            id,
        }
    }

    fn id(&self) -> Option<StoreId> {
        Some(self.id)
    }

    fn into_bare(self) -> Self::BareTy {
        self.inner
    }
}

impl<T: Clone> Clone for Stored<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            inner: self.inner.clone(),
        }
    }
}

impl<T: Copy> Copy for Stored<T> {}

impl<T: core::fmt::Debug> core::fmt::Debug for Stored<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Stored")
            .field("inner", &self)
            .field("id", &self.id)
            .finish()
    }
}

impl<T: PartialEq> PartialEq for Stored<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id) && self.inner.eq(&other.inner)
    }
}

impl<T: Eq> Eq for Stored<T> {}

impl<T> Deref for Stored<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for Stored<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// A stored variant of [`ExternVal`]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum StoredExternVal {
    Func(Stored<FuncAddr>),
    Table(Stored<TableAddr>),
    Mem(Stored<MemAddr>),
    Global(Stored<GlobalAddr>),
}

impl AbstractStored for StoredExternVal {
    type BareTy = ExternVal;

    unsafe fn from_bare(bare_value: Self::BareTy, id: StoreId) -> Self {
        match bare_value {
            ExternVal::Func(func_addr) => {
                // SAFETY: Upheld by the caller
                let stored_func_addr = unsafe { Stored::from_bare(func_addr, id) };
                Self::Func(stored_func_addr)
            }
            ExternVal::Table(table_addr) => {
                // SAFETY: Upheld by the caller
                let stored_table_addr = unsafe { Stored::from_bare(table_addr, id) };
                Self::Table(stored_table_addr)
            }
            ExternVal::Mem(mem_addr) => {
                // SAFETY: Upheld by the caller
                let stored_mem_addr = unsafe { Stored::from_bare(mem_addr, id) };
                Self::Mem(stored_mem_addr)
            }
            ExternVal::Global(global_addr) => {
                // SAFETY: Upheld by the caller
                let stored_global_addr = unsafe { Stored::from_bare(global_addr, id) };
                Self::Global(stored_global_addr)
            }
        }
    }

    fn id(&self) -> Option<StoreId> {
        match self {
            StoredExternVal::Func(stored_func_addr) => stored_func_addr.id(),
            StoredExternVal::Table(stored_table_addr) => stored_table_addr.id(),
            StoredExternVal::Mem(stored_mem_addr) => stored_mem_addr.id(),
            StoredExternVal::Global(stored_global_addr) => stored_global_addr.id(),
        }
    }

    fn into_bare(self) -> Self::BareTy {
        match self {
            StoredExternVal::Func(stored_func_addr) => {
                ExternVal::Func(stored_func_addr.into_bare())
            }
            StoredExternVal::Table(stored_table_addr) => {
                ExternVal::Table(stored_table_addr.into_bare())
            }
            StoredExternVal::Mem(stored_mem_addr) => ExternVal::Mem(stored_mem_addr.into_bare()),
            StoredExternVal::Global(stored_global_addr) => {
                ExternVal::Global(stored_global_addr.into_bare())
            }
        }
    }
}

impl StoredExternVal {
    pub fn as_func(self) -> Option<Stored<FuncAddr>> {
        match self {
            StoredExternVal::Func(func_addr) => Some(func_addr),
            _ => None,
        }
    }

    pub fn as_table(self) -> Option<Stored<TableAddr>> {
        match self {
            StoredExternVal::Table(table_addr) => Some(table_addr),
            _ => None,
        }
    }

    pub fn as_mem(self) -> Option<Stored<MemAddr>> {
        match self {
            StoredExternVal::Mem(mem_addr) => Some(mem_addr),
            _ => None,
        }
    }

    pub fn as_global(self) -> Option<Stored<GlobalAddr>> {
        match self {
            StoredExternVal::Global(global_addr) => Some(global_addr),
            _ => None,
        }
    }
}

/// A stored variant of [`Resumable`]
pub enum StoredResumable {
    Wasm(Stored<WasmResumable>),
    Host(Stored<HostThing>),
}

impl AbstractStored for StoredResumable {
    type BareTy = Resumable;

    unsafe fn from_bare(bare_value: Self::BareTy, id: StoreId) -> Self {
        match bare_value {
            Resumable::Wasm(wasm_resumable) => {
                // SAFETY: Upheld by caller
                Self::Wasm(unsafe { Stored::from_bare(wasm_resumable, id) })
            }
            Resumable::Host(host_thing) => {
                // SAFETY: Upheld by caller
                Self::Host(unsafe { Stored::from_bare(host_thing, id) })
            }
        }
    }

    fn id(&self) -> Option<StoreId> {
        match self {
            Self::Wasm(wasm_resumable) => wasm_resumable.id(),
            Self::Host(host_thing) => host_thing.id(),
        }
    }

    fn into_bare(self) -> Self::BareTy {
        match self {
            Self::Wasm(wasm_resumable) => Resumable::Wasm(wasm_resumable.into_bare()),
            Self::Host(host_thing) => Resumable::Host(host_thing.into_bare()),
        }
    }
}

/// A stored variant of [`HostCall`]
pub struct StoredHostCall {
    /// Must contain the correct parameter types for the host function with host
    /// code `hostcode`.
    pub params: Vec<StoredValue>,
    pub hostcode: Hostcode,
}

impl AbstractStored for StoredHostCall {
    type BareTy = HostCall;

    unsafe fn from_bare(bare_value: Self::BareTy, id: StoreId) -> Self {
        Self {
            // SAFETY: Upheld by caller
            params: unsafe { Vec::from_bare(bare_value.params, id) },
            hostcode: bare_value.hostcode,
        }
    }

    fn id(&self) -> Option<StoreId> {
        self.params.id()
    }

    fn into_bare(self) -> Self::BareTy {
        HostCall {
            params: self.params.into_bare(),
            hostcode: self.hostcode,
        }
    }

    fn try_unwrap_into_bare(self, expected_store_id: StoreId) -> Self::BareTy {
        HostCall {
            params: self.params.try_unwrap_into_bare(expected_store_id),
            hostcode: self.hostcode,
        }
    }
}

/// A stored variant of [`RunState`]
pub enum StoredRunState {
    Finished {
        values: Vec<StoredValue>,
        maybe_remaining_fuel: Option<u64>,
    },
    Resumable {
        resumable: Stored<WasmResumable>,
        required_fuel: Option<NonZeroU64>,
    },
    HostCalled {
        host_call: StoredHostCall,
        resumable: Stored<HostResumable>,
    },
}

impl AbstractStored for StoredRunState {
    type BareTy = RunState;

    unsafe fn from_bare(bare_value: Self::BareTy, id: StoreId) -> Self {
        match bare_value {
            RunState::Finished {
                values,
                maybe_remaining_fuel,
            } => Self::Finished {
                // SAFETY: Upheld by the caller
                values: unsafe { Vec::from_bare(values, id) },
                maybe_remaining_fuel,
            },
            RunState::Resumable {
                resumable,
                required_fuel,
            } => Self::Resumable {
                // SAFETY: Upheld by the caller
                resumable: unsafe { Stored::from_bare(resumable, id) },
                required_fuel,
            },
            RunState::HostCalled {
                host_call,
                resumable,
            } => Self::HostCalled {
                // SAFETY: Upheld by caller
                host_call: unsafe { StoredHostCall::from_bare(host_call, id) },
                // SAFETY: Upheld by caller
                resumable: unsafe { Stored::from_bare(resumable, id) },
            },
        }
    }

    fn id(&self) -> Option<StoreId> {
        todo!()
    }

    fn into_bare(self) -> Self::BareTy {
        todo!()
    }
}

/// A stored variant of [`InstantiationOutcome`]
pub struct StoredInstantiationOutcome {
    pub module_addr: Stored<ModuleAddr>,
    pub maybe_remaining_fuel: Option<u64>,
}

impl AbstractStored for StoredInstantiationOutcome {
    type BareTy = InstantiationOutcome;

    unsafe fn from_bare(bare_value: Self::BareTy, id: StoreId) -> Self {
        Self {
            // SAFETY: Upheld by the caller
            module_addr: unsafe { Stored::from_bare(bare_value.module_addr, id) },
            maybe_remaining_fuel: bare_value.maybe_remaining_fuel,
        }
    }

    fn id(&self) -> Option<StoreId> {
        self.module_addr.id()
    }

    fn into_bare(self) -> Self::BareTy {
        InstantiationOutcome {
            module_addr: self.module_addr.into_bare(),
            maybe_remaining_fuel: self.maybe_remaining_fuel,
        }
    }
}
