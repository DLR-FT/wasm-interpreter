//! TODO: Logic that makes sure that objects, which belong to a specific [`Store`], are only used with that [`Store`].

use core::{
    num::NonZeroU32,
    sync::atomic::{AtomicU64, Ordering},
};

use alloc::vec::Vec;

use crate::{
    config::Config,
    core::reader::types::{FuncType, MemType, TableType},
    resumable::{ResumableRef, RunState},
    value::{ExternAddr, Ref, F32, F64},
    GlobalType, RefType, RuntimeError, RuntimeInstance, ValidationInfo, Value,
};

use super::{
    addrs::{FuncAddr, GlobalAddr, MemAddr, ModuleAddr, TableAddr},
    ExternVal, Store,
};

/// A wrapper around a [`Store`], that forwards all method calls to the inner
/// [`Store`] while performing safety checks at runtime.
///
/// TODO more docs
pub struct CheckedStore<'b, T: Config> {
    /// The inner [`Store`]
    inner: Store<'b, T>,

    /// A unique identifier for this store. This is used to verify that
    /// reference/address types target this specific [`Store`].
    id: StoreId,
}

/// A unique identifier for a specfic [`Store`]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct StoreId(u64);

impl StoreId {
    /// Creates a new unique [`StoreId`]
    #[allow(clippy::new_without_default)] // reason = "StoreId::default() might be misunderstood to be some default value. However, a
                                          // default value does not exist in that sense because every newly created StoreId must be unique"
    pub fn new() -> Self {
        static NEXT_STORE_ID: AtomicU64 = AtomicU64::new(0);

        // TODO find a fix for the default wrapping behaviour of `fetch_add`.
        // Maybe we could return a `RuntimeError` here?
        Self(NEXT_STORE_ID.fetch_add(1, Ordering::SeqCst))
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Stored<T> {
    id: StoreId,
    inner: T,
}

impl<T> Stored<T> {
    fn as_mut(&mut self) -> Stored<&mut T> {
        Stored {
            id: self.id,
            inner: &mut self.inner,
        }
    }
}

pub enum StoredValue {
    I32(u32),
    I64(u64),
    F32(F32),
    F64(F64),
    V128([u8; 16]),
    Ref(StoredRef),
}

#[derive(Clone, Copy, Debug)]
pub enum StoredRef {
    Null(RefType),
    Func(Stored<FuncAddr>),
    Extern(Stored<ExternAddr>),
}

/// The stored variant of [`ExternVal`]
#[derive(Debug, Copy, Clone)]
pub enum StoredExternVal {
    Func(Stored<FuncAddr>),
    Table(Stored<TableAddr>),
    Mem(Stored<MemAddr>),
    Global(Stored<GlobalAddr>),
}

/// The stored variant of [`RunState`]
pub enum StoredRunState {
    Finished(Vec<StoredValue>),
    Resumable {
        resumable_ref: Stored<ResumableRef>,
        required_fuel: NonZeroU32,
    },
}

impl<T: Config> CheckedStore<'_, T> {
    pub(crate) fn wrap_stored<U>(&self, inner: U) -> Stored<U> {
        Stored { id: self.id, inner }
    }

    pub(crate) fn try_unwrap_stored<U>(&self, stored: Stored<U>) -> Result<U, RuntimeError> {
        if self.id == stored.id {
            Ok(stored.inner)
        } else {
            Err(RuntimeError::StoreIdMismatch)
        }
    }

    pub(crate) fn wrap_value(&self, value: Value) -> StoredValue {
        match value {
            Value::I32(x) => StoredValue::I32(x),
            Value::I64(x) => StoredValue::I64(x),
            Value::F32(x) => StoredValue::F32(x),
            Value::F64(x) => StoredValue::F64(x),
            Value::V128(x) => StoredValue::V128(x),
            Value::Ref(r#ref) => StoredValue::Ref(self.wrap_ref(r#ref)),
        }
    }

    pub(crate) fn try_unwrap_value(
        &self,
        stored_value: StoredValue,
    ) -> Result<Value, RuntimeError> {
        let value = match stored_value {
            StoredValue::I32(x) => Value::I32(x),
            StoredValue::I64(x) => Value::I64(x),
            StoredValue::F32(x) => Value::F32(x),
            StoredValue::F64(x) => Value::F64(x),
            StoredValue::V128(x) => Value::V128(x),
            StoredValue::Ref(stored_ref) => Value::Ref(self.try_unwrap_ref(stored_ref)?),
        };

        Ok(value)
    }

    pub(crate) fn wrap_ref(&self, r#ref: Ref) -> StoredRef {
        match r#ref {
            Ref::Null(ref_type) => StoredRef::Null(ref_type),
            Ref::Func(func_addr) => StoredRef::Func(self.wrap_stored(func_addr)),
            Ref::Extern(extern_addr) => StoredRef::Extern(self.wrap_stored(extern_addr)),
        }
    }

    pub(crate) fn try_unwrap_ref(&self, stored_ref: StoredRef) -> Result<Ref, RuntimeError> {
        let r#ref = match stored_ref {
            StoredRef::Null(ref_type) => Ref::Null(ref_type),
            StoredRef::Func(func_addr) => Ref::Func(self.try_unwrap_stored(func_addr)?),
            StoredRef::Extern(extern_addr) => Ref::Extern(self.try_unwrap_stored(extern_addr)?),
        };
        Ok(r#ref)
    }

    pub(crate) fn wrap_extern_val(&self, extern_val: ExternVal) -> StoredExternVal {
        match extern_val {
            ExternVal::Func(addr) => StoredExternVal::Func(self.wrap_stored(addr)),
            ExternVal::Table(addr) => StoredExternVal::Table(self.wrap_stored(addr)),
            ExternVal::Mem(addr) => StoredExternVal::Mem(self.wrap_stored(addr)),
            ExternVal::Global(addr) => StoredExternVal::Global(self.wrap_stored(addr)),
        }
    }

    pub(crate) fn wrap_run_state(&self, run_state: RunState) -> StoredRunState {
        match run_state {
            RunState::Finished(values) => StoredRunState::Finished(
                values
                    .into_iter()
                    .map(|value| self.wrap_value(value))
                    .collect(),
            ),
            RunState::Resumable {
                resumable_ref,
                required_fuel,
            } => StoredRunState::Resumable {
                resumable_ref: self.wrap_stored(resumable_ref),
                required_fuel,
            },
        }
    }
}

impl<'b, T: Config> CheckedStore<'b, T> {
    pub fn new(user_data: T) -> Self {
        Self {
            inner: Store::new(user_data),
            id: StoreId::new(),
        }
    }

    /// Instantiates a validated module with `validation_info` as validation evidence with name `name`.
    ///
    /// This is the safe variant of [`Store::add_module`]. See
    /// [`CheckedStore`] for more information.
    pub fn add_module(
        &mut self,
        name: &str,
        validation_info: &ValidationInfo<'b>,
        maybe_fuel: Option<u32>,
    ) -> Result<Stored<ModuleAddr>, RuntimeError> {
        self.inner
            .add_module(name, validation_info, maybe_fuel)
            .map(|module_addr| self.wrap_stored(module_addr))
    }

    /// Gets an export of a specific module instance by its name
    ///
    /// This is the safe variant of [`Store::instance_export`]. See
    /// [`CheckedStore`] for more information.
    pub fn instance_export(
        &self,
        module_addr: Stored<ModuleAddr>,
        name: &str,
    ) -> Result<StoredExternVal, RuntimeError> {
        let module_addr = self.try_unwrap_stored(module_addr)?;
        self.inner
            .instance_export(module_addr, name)
            .map(|extern_val| self.wrap_extern_val(extern_val))
    }

    // Note: `pub fn func_alloc(&mut self, ...)` is missing, because it would
    // require changes in the core interpreter.

    /// Gets the type of a function by its addr.
    ///
    /// This is the safe variant of [`Store::func_type`]. See [`CheckedStore`]
    /// for more information.
    pub fn func_type(&self, func_addr: Stored<FuncAddr>) -> Result<FuncType, RuntimeError> {
        let func_addr = self.try_unwrap_stored(func_addr)?;
        Ok(self.inner.func_type(func_addr))
    }

    /// This is the safe variant of [`Store::invoke`]. See [`CheckedStore`] for
    /// more information.
    pub fn invoke(
        &mut self,
        func_addr: Stored<FuncAddr>,
        params: Vec<StoredValue>,
        maybe_fuel: Option<u32>,
    ) -> Result<StoredRunState, RuntimeError> {
        let func_addr = self.try_unwrap_stored(func_addr)?;
        let params = params
            .into_iter()
            .map(|value| self.try_unwrap_value(value))
            .collect::<Result<Vec<Value>, RuntimeError>>()?;

        self.inner
            .invoke(func_addr, params, maybe_fuel)
            .map(|run_state| self.wrap_run_state(run_state))
    }

    /// This is the safe variant of [`Store::table_alloc`]. See [`CheckedStore`] for
    /// more information.
    pub fn table_alloc(
        &mut self,
        table_type: TableType,
        r#ref: StoredRef,
    ) -> Result<Stored<TableAddr>, RuntimeError> {
        let r#ref = self.try_unwrap_ref(r#ref)?;
        self.inner
            .table_alloc(table_type, r#ref)
            .map(|table_addr| self.wrap_stored(table_addr))
    }

    /// This is the safe variant of [`Store::table_type`]. See [`CheckedStore`] for
    /// more information.
    pub fn table_type(&self, table_addr: Stored<TableAddr>) -> Result<TableType, RuntimeError> {
        let table_addr = self.try_unwrap_stored(table_addr)?;
        Ok(self.inner.table_type(table_addr))
    }

    /// This is the safe variant of [`Store::table_read`]. See [`CheckedStore`] for
    /// more information.
    pub fn table_read(
        &self,
        table_addr: Stored<TableAddr>,
        i: u32,
    ) -> Result<StoredRef, RuntimeError> {
        let table_addr = self.try_unwrap_stored(table_addr)?;
        self.inner
            .table_read(table_addr, i)
            .map(|r#ref| self.wrap_ref(r#ref))
    }

    /// This is the safe variant of [`Store::table_write`]. See [`CheckedStore`] for
    /// more information.
    pub fn table_write(
        &mut self,
        table_addr: Stored<TableAddr>,
        i: u32,
        r#ref: StoredRef,
    ) -> Result<(), RuntimeError> {
        let table_addr = self.try_unwrap_stored(table_addr)?;
        let r#ref = self.try_unwrap_ref(r#ref)?;
        self.inner.table_write(table_addr, i, r#ref)
    }

    /// This is the safe variant of [`Store::table_size`]. See [`CheckedStore`] for
    /// more information.
    pub fn table_size(&self, table_addr: Stored<TableAddr>) -> Result<u32, RuntimeError> {
        let table_addr = self.try_unwrap_stored(table_addr)?;

        Ok(self.inner.table_size(table_addr))
    }

    /// This is the safe variant of [`Store::mem_alloc`]. See [`CheckedStore`] for
    /// more information.
    pub fn mem_alloc(&mut self, mem_type: MemType) -> Stored<MemAddr> {
        let mem_addr = self.inner.mem_alloc(mem_type);
        self.wrap_stored(mem_addr)
    }

    /// This is the safe variant of [`Store::mem_type`]. See [`CheckedStore`] for
    /// more information.
    pub fn mem_type(&self, mem_addr: Stored<MemAddr>) -> Result<MemType, RuntimeError> {
        let mem_addr = self.try_unwrap_stored(mem_addr)?;
        Ok(self.inner.mem_type(mem_addr))
    }

    /// This is the safe variant of [`Store::mem_read`]. See [`CheckedStore`] for
    /// more information.
    pub fn mem_read(&self, mem_addr: Stored<MemAddr>, i: u32) -> Result<u8, RuntimeError> {
        let mem_addr = self.try_unwrap_stored(mem_addr)?;
        self.inner.mem_read(mem_addr, i)
    }

    /// This is the safe variant of [`Store::mem_write`]. See [`CheckedStore`] for
    /// more information.
    pub fn mem_write(
        &mut self,
        mem_addr: Stored<MemAddr>,
        i: u32,
        byte: u8,
    ) -> Result<(), RuntimeError> {
        let mem_addr = self.try_unwrap_stored(mem_addr)?;

        self.inner.mem_write(mem_addr, i, byte)
    }

    /// This is the safe variant of [`Store::mem_size`]. See [`CheckedStore`] for
    /// more information.
    pub fn mem_size(&self, mem_addr: Stored<MemAddr>) -> Result<u32, RuntimeError> {
        let mem_addr = self.try_unwrap_stored(mem_addr)?;
        Ok(self.inner.mem_size(mem_addr))
    }

    /// This is the safe variant of [`Store::mem_grow`]. See [`CheckedStore`] for
    /// more information.
    pub fn mem_grow(&mut self, mem_addr: Stored<MemAddr>, n: u32) -> Result<(), RuntimeError> {
        let mem_addr = self.try_unwrap_stored(mem_addr)?;
        self.inner.mem_grow(mem_addr, n)
    }

    /// This is the safe variant of [`Store::global_alloc`]. See [`CheckedStore`] for
    /// more information.
    pub fn global_alloc(
        &mut self,
        global_type: GlobalType,
        val: StoredValue,
    ) -> Result<Stored<GlobalAddr>, RuntimeError> {
        let val = self.try_unwrap_value(val)?;
        self.inner
            .global_alloc(global_type, val)
            .map(|global_addr| self.wrap_stored(global_addr))
    }

    /// This is the safe variant of [`Store::global_type`]. See [`CheckedStore`] for
    /// more information.
    pub fn global_type(&self, global_addr: Stored<GlobalAddr>) -> Result<GlobalType, RuntimeError> {
        let global_addr = self.try_unwrap_stored(global_addr)?;
        Ok(self.inner.global_type(global_addr))
    }

    /// This is the safe variant of [`Store::global_read`]. See [`CheckedStore`] for
    /// more information.
    pub fn global_read(
        &self,
        global_addr: Stored<GlobalAddr>,
    ) -> Result<StoredValue, RuntimeError> {
        let global_addr = self.try_unwrap_stored(global_addr)?;
        let value = self.inner.global_read(global_addr);
        Ok(self.wrap_value(value))
    }

    /// This is the safe variant of [`Store::global_write`]. See [`CheckedStore`] for
    /// more information.
    pub fn global_write(
        &mut self,
        global_addr: Stored<GlobalAddr>,
        val: StoredValue,
    ) -> Result<(), RuntimeError> {
        let global_addr = self.try_unwrap_stored(global_addr)?;
        let val = self.try_unwrap_value(val)?;
        self.inner.global_write(global_addr, val)
    }

    /// This is the safe variant of [`Store::reregister_module`]. See [`CheckedStore`] for
    /// more information.
    pub fn reregister_module(
        &mut self,
        module_addr: Stored<ModuleAddr>,
        name: &str,
    ) -> Result<(), RuntimeError> {
        let module_addr = self.try_unwrap_stored(module_addr)?;
        self.inner.reregister_module(module_addr, name)
    }

    /// This is the safe variant of [`Store::create_resumable`]. See [`CheckedStore`] for
    /// more information.
    pub fn create_resumable(
        &self,
        func_addr: Stored<FuncAddr>,
        params: Vec<StoredValue>,
        maybe_fuel: Option<u32>,
    ) -> Result<Stored<ResumableRef>, RuntimeError> {
        let func_addr = self.try_unwrap_stored(func_addr)?;
        let params = params
            .into_iter()
            .map(|param| self.try_unwrap_value(param))
            .collect::<Result<Vec<Value>, RuntimeError>>()?;
        let resumable_ref = self.inner.create_resumable(func_addr, params, maybe_fuel)?;
        Ok(self.wrap_stored(resumable_ref))
    }

    /// This is the safe variant of [`Store::resume`]. See [`CheckedStore`] for
    /// more information.
    pub fn resume(
        &mut self,
        resumable_ref: Stored<ResumableRef>,
    ) -> Result<StoredRunState, RuntimeError> {
        let resumable_ref = self.try_unwrap_stored(resumable_ref)?;
        let run_state = self.inner.resume(resumable_ref)?;
        Ok(self.wrap_run_state(run_state))
    }

    /// This is the safe variant of [`Store::access_fuel_mut`]. See [`CheckedStore`] for
    /// more information.
    // TODO `&mut Stored<...>` seems off as a parameter type
    pub fn access_fuel_mut<R>(
        &mut self,
        resumable_ref: &mut Stored<ResumableRef>,
        f: impl FnOnce(&mut Option<u32>) -> R,
    ) -> Result<R, RuntimeError> {
        let resumable_ref = self.try_unwrap_stored(resumable_ref.as_mut())?;
        self.inner.access_fuel_mut(resumable_ref, f)
    }
}
