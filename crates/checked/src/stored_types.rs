use core::{num::NonZeroU64, ops::Deref};

use alloc::vec::Vec;
use wasm::{
    addrs::{FuncAddr, GlobalAddr, MemAddr, ModuleAddr, TableAddr},
    resumable::{Resumable, RunState},
    ExternVal, InstantiationOutcome,
};

use crate::{AbstractStored, StoreId, StoredValue};

/// A generic stored wrapper. This is used to wrap `struct` types such as
/// [`FuncAddr`], [`MemAddr`], etc.
pub struct Stored<T> {
    id: StoreId,
    /// The inner value of this stored object.
    ///
    /// # Safety
    ///
    /// It is important that mutable access to the this inner value is not
    /// exposed to the user directly. Currently, there exists one exception to
    /// this rule: [`Stored<Resumable<T>>::fuel_mut`].
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

    fn into_bare(self) -> Self::BareTy {
        self.inner
    }

    fn try_unwrap_into_bare(self, expected_store_id: StoreId) -> Self::BareTy {
        if self.id != expected_store_id {
            panic!("Mismatched store ids");
        }

        self.into_bare()
    }
}

impl<T> Stored<T> {
    pub(crate) fn id(&self) -> StoreId {
        self.id
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

// Unfortuately we cannot implement `DerefMut` for `Stored`, because that allows
// the user to replace the inner T. Therefore, wrap this method manually.
impl<T> Stored<Resumable<T>> {
    pub fn fuel_mut(&mut self) -> &mut Option<u64> {
        self.inner.fuel_mut()
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

    fn try_unwrap_into_bare(self, expected_store_id: StoreId) -> Self::BareTy {
        match self {
            StoredExternVal::Func(stored_func_addr) => {
                ExternVal::Func(stored_func_addr.try_unwrap_into_bare(expected_store_id))
            }
            StoredExternVal::Table(stored_table_addr) => {
                ExternVal::Table(stored_table_addr.try_unwrap_into_bare(expected_store_id))
            }
            StoredExternVal::Mem(stored_mem_addr) => {
                ExternVal::Mem(stored_mem_addr.try_unwrap_into_bare(expected_store_id))
            }
            StoredExternVal::Global(stored_global_addr) => {
                ExternVal::Global(stored_global_addr.try_unwrap_into_bare(expected_store_id))
            }
        }
    }
}

impl StoredExternVal {
    pub(crate) fn id(&self) -> StoreId {
        match self {
            StoredExternVal::Func(stored) => stored.id,
            StoredExternVal::Table(stored) => stored.id,
            StoredExternVal::Mem(stored) => stored.id,
            StoredExternVal::Global(stored) => stored.id,
        }
    }

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

/// A stored variant of [`RunState`]
pub enum StoredRunState<T> {
    Finished {
        values: Vec<StoredValue>,
        maybe_remaining_fuel: Option<u64>,
    },
    Resumable {
        resumable: Stored<Resumable<T>>,
        required_fuel: NonZeroU64,
    },
}

impl<T> AbstractStored for StoredRunState<T> {
    type BareTy = RunState<T>;

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
        }
    }

    fn into_bare(self) -> Self::BareTy {
        match self {
            StoredRunState::Finished {
                values,
                maybe_remaining_fuel,
            } => RunState::Finished {
                values: values.into_bare(),
                maybe_remaining_fuel,
            },
            StoredRunState::Resumable {
                resumable,
                required_fuel,
            } => RunState::Resumable {
                resumable: resumable.into_bare(),
                required_fuel,
            },
        }
    }

    fn try_unwrap_into_bare(self, expected_store_id: StoreId) -> Self::BareTy {
        match self {
            StoredRunState::Finished {
                values,
                maybe_remaining_fuel,
            } => RunState::Finished {
                values: values.try_unwrap_into_bare(expected_store_id),
                maybe_remaining_fuel,
            },
            StoredRunState::Resumable {
                resumable,
                required_fuel,
            } => RunState::Resumable {
                resumable: resumable.try_unwrap_into_bare(expected_store_id),
                required_fuel,
            },
        }
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

    fn into_bare(self) -> Self::BareTy {
        InstantiationOutcome {
            module_addr: self.module_addr.into_bare(),
            maybe_remaining_fuel: self.maybe_remaining_fuel,
        }
    }

    fn try_unwrap_into_bare(self, expected_store_id: StoreId) -> Self::BareTy {
        InstantiationOutcome {
            module_addr: self.module_addr.try_unwrap_into_bare(expected_store_id),
            maybe_remaining_fuel: self.maybe_remaining_fuel,
        }
    }
}
