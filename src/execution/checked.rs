//! Definitions for checked, safe variants of methods defined on [`Store`].
//!
//! All methods are simply wrappers which ensure that the addresses used belong to the current store.

use core::{
    num::NonZeroU32,
    sync::atomic::{AtomicU64, Ordering},
};

use alloc::vec::Vec;

use crate::{
    addrs::{FuncAddr, GlobalAddr, MemAddr, ModuleAddr, TableAddr},
    config::Config,
    core::reader::types::{FuncType, MemType, TableType},
    resumable::{ResumableRef, RunState},
    value::{ExternAddr, Ref, F32, F64},
    GlobalType, RefType, RuntimeError, ValidationInfo, Value,
};

use super::{ExternVal, Store};

/// A unique identifier for a specific [`Store`]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct StoreId(u64);

impl StoreId {
    /// Creates a new unique [`StoreId`]
    #[allow(clippy::new_without_default)] // reason = "StoreId::default() might be misunderstood to be some
                                          // default value. However, a default value does not exist in that
                                          // sense because every newly created StoreId must be unique"
    pub fn new() -> Self {
        static NEXT_STORE_ID: AtomicU64 = AtomicU64::new(0);

        // TODO find a fix for the default wrapping behavior of `fetch_add`.
        // Maybe we could return a `RuntimeError` here?
        Self(NEXT_STORE_ID.fetch_add(1, Ordering::SeqCst))
    }
}

impl<T: Config> Store<'_, T> {
    /// Associates some value of type `U` with the current store, producing a
    /// [`Stored<U>`] object. This object can then be unwrapped later.
    ///
    /// See also: [`Store::try_unwrap_stored`].
    fn wrap_stored<U>(&self, inner: U) -> Stored<U> {
        Stored { id: self.id, inner }
    }

    /// Validates that some [`Stored<U>`] object comes from the current
    /// [`Store`]. If this is true, the inner value of type `U` is
    /// returned, otherwise an error is returned.
    ///
    /// See also: [`Store::wrap_stored`].
    ///
    /// # Errors
    /// - [`RuntimeError::StoreIdMismatch`]
    pub(crate) fn try_unwrap_stored<U>(&self, stored: Stored<U>) -> Result<U, RuntimeError> {
        if self.id == stored.id {
            Ok(stored.inner)
        } else {
            Err(RuntimeError::StoreIdMismatch)
        }
    }

    /// Associates some [`Value`] with the current [`Store`], producing a
    /// [`StoredValue`] object. This object can be matched against or unwrapped
    /// later.
    ///
    /// See also: [`Store::try_unwrap_value`].
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

    /// Validates that some [`StoredValue`] comes from the current
    /// [`Store`]. If this is true, the value is returned as a [`Value`],
    /// otherwise an error is returned.
    ///
    /// See also: [`Store::wrap_value`].
    ///
    /// # Errors
    /// - [`RuntimeError::StoreIdMismatch`]
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

    /// Associates some [`Ref`] with the current [`Store`], producing a
    /// [`StoredRef`] object. This object can be matched against or unwrapped
    /// later.
    ///
    /// See also: [`Store::try_unwrap_ref`].
    pub(crate) fn wrap_ref(&self, r#ref: Ref) -> StoredRef {
        match r#ref {
            Ref::Null(ref_type) => StoredRef::Null(ref_type),
            Ref::Func(func_addr) => StoredRef::Func(self.wrap_stored(func_addr)),
            Ref::Extern(extern_addr) => StoredRef::Extern(self.wrap_stored(extern_addr)),
        }
    }

    /// Validates that some [`StoredRef`] comes from the current
    /// [`Store`]. If this is true, the value is returned as a [`Ref`],
    /// otherwise an error is returned.
    ///
    /// See also: [`Store::wrap_ref`].
    ///
    /// # Errors
    /// - [`RuntimeError::StoreIdMismatch`]
    pub(crate) fn try_unwrap_ref(&self, stored_ref: StoredRef) -> Result<Ref, RuntimeError> {
        let r#ref = match stored_ref {
            StoredRef::Null(ref_type) => Ref::Null(ref_type),
            StoredRef::Func(func_addr) => Ref::Func(self.try_unwrap_stored(func_addr)?),
            StoredRef::Extern(extern_addr) => Ref::Extern(self.try_unwrap_stored(extern_addr)?),
        };
        Ok(r#ref)
    }

    /// Associates some [`ExternVal`] with the current [`Store`],
    /// producing a [`StoredExternVal`] object. This object can only be matched
    /// against to obtain different kinds of [`Stored<...>`] objects.
    ///
    /// See also: [`Store::try_unwrap_stored`].
    pub(crate) fn wrap_extern_val(&self, extern_val: ExternVal) -> StoredExternVal {
        match extern_val {
            ExternVal::Func(addr) => StoredExternVal::Func(self.wrap_stored(addr)),
            ExternVal::Table(addr) => StoredExternVal::Table(self.wrap_stored(addr)),
            ExternVal::Mem(addr) => StoredExternVal::Mem(self.wrap_stored(addr)),
            ExternVal::Global(addr) => StoredExternVal::Global(self.wrap_stored(addr)),
        }
    }

    pub(crate) fn try_unwrap_extern_val(
        &self,
        stored_extern_val: StoredExternVal,
    ) -> Result<ExternVal, RuntimeError> {
        match stored_extern_val {
            StoredExternVal::Func(func_addr) => {
                self.try_unwrap_stored(func_addr).map(ExternVal::Func)
            }
            StoredExternVal::Table(table_addr) => {
                self.try_unwrap_stored(table_addr).map(ExternVal::Table)
            }
            StoredExternVal::Mem(mem_addr) => self.try_unwrap_stored(mem_addr).map(ExternVal::Mem),
            StoredExternVal::Global(global_addr) => {
                self.try_unwrap_stored(global_addr).map(ExternVal::Global)
            }
        }
    }

    /// Associates some [`ExternVal`] with the current [`Store`],
    /// producing a [`StoredExternVal`] object. This object can only be matched
    /// against to obtain [`Stored<...>`] or [`StoredValue`] objects.
    ///
    /// See also: [`Store::try_unwrap_stored`], [`Store::try_unwrap_value`].
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

impl<'b, T: Config> Store<'b, T> {
    /// This is a safe variant of [`Store::module_instantiate`].
    pub fn module_instantiate_checked(
        &mut self,
        validation_info: &ValidationInfo<'b>,
        extern_vals: Vec<StoredExternVal>,
        maybe_fuel: Option<u32>,
    ) -> Result<Stored<ModuleAddr>, RuntimeError> {
        let extern_vals = extern_vals
            .into_iter()
            .map(|extern_val| self.try_unwrap_extern_val(extern_val))
            .collect::<Result<Vec<ExternVal>, RuntimeError>>()?;

        self.module_instantiate(validation_info, extern_vals, maybe_fuel)
            .map(|module_addr| self.wrap_stored(module_addr))
    }

    /// This is a safe variant of [`Store::instance_export`].
    pub fn instance_export_checked(
        &self,
        module_addr: Stored<ModuleAddr>,
        name: &str,
    ) -> Result<StoredExternVal, RuntimeError> {
        let module_addr = self.try_unwrap_stored(module_addr)?;
        self.instance_export(module_addr, name)
            .map(|extern_val| self.wrap_extern_val(extern_val))
    }

    // Note: `pub fn func_alloc(&mut self, ...)` is missing, because it would
    // require changes in the core interpreter.

    /// This is a safe variant of [`Store::func_type`].
    pub fn func_type_checked(&self, func_addr: Stored<FuncAddr>) -> Result<FuncType, RuntimeError> {
        let func_addr = self.try_unwrap_stored(func_addr)?;
        Ok(self.func_type(func_addr))
    }

    /// This is a safe variant of [`Store::invoke`].
    pub fn invoke_checked(
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

        self.invoke(func_addr, params, maybe_fuel)
            .map(|run_state| self.wrap_run_state(run_state))
    }

    /// This is a safe variant of [`Store::table_alloc`].
    pub fn table_alloc_checked(
        &mut self,
        table_type: TableType,
        r#ref: StoredRef,
    ) -> Result<Stored<TableAddr>, RuntimeError> {
        let r#ref = self.try_unwrap_ref(r#ref)?;
        self.table_alloc(table_type, r#ref)
            .map(|table_addr| self.wrap_stored(table_addr))
    }

    /// This is a safe variant of [`Store::table_type`].
    pub fn table_type_checked(
        &self,
        table_addr: Stored<TableAddr>,
    ) -> Result<TableType, RuntimeError> {
        let table_addr = self.try_unwrap_stored(table_addr)?;
        Ok(self.table_type(table_addr))
    }

    /// This is a safe variant of [`Store::table_read`].
    pub fn table_read_checked(
        &self,
        table_addr: Stored<TableAddr>,
        i: u32,
    ) -> Result<StoredRef, RuntimeError> {
        let table_addr = self.try_unwrap_stored(table_addr)?;
        self.table_read(table_addr, i)
            .map(|r#ref| self.wrap_ref(r#ref))
    }

    /// This is a safe variant of [`Store::table_write`].
    pub fn table_write_checked(
        &mut self,
        table_addr: Stored<TableAddr>,
        i: u32,
        r#ref: StoredRef,
    ) -> Result<(), RuntimeError> {
        let table_addr = self.try_unwrap_stored(table_addr)?;
        let r#ref = self.try_unwrap_ref(r#ref)?;
        self.table_write(table_addr, i, r#ref)
    }

    /// This is a safe variant of [`Store::table_size`].
    pub fn table_size_checked(&self, table_addr: Stored<TableAddr>) -> Result<u32, RuntimeError> {
        let table_addr = self.try_unwrap_stored(table_addr)?;

        Ok(self.table_size(table_addr))
    }

    /// This is a safe variant of [`Store::mem_alloc`].
    pub fn mem_alloc_checked(&mut self, mem_type: MemType) -> Stored<MemAddr> {
        let mem_addr = self.mem_alloc(mem_type);
        self.wrap_stored(mem_addr)
    }

    /// This is a safe variant of [`Store::mem_type`].
    pub fn mem_type_checked(&self, mem_addr: Stored<MemAddr>) -> Result<MemType, RuntimeError> {
        let mem_addr = self.try_unwrap_stored(mem_addr)?;
        Ok(self.mem_type(mem_addr))
    }

    /// This is a safe variant of [`Store::mem_read`].
    pub fn mem_read_checked(&self, mem_addr: Stored<MemAddr>, i: u32) -> Result<u8, RuntimeError> {
        let mem_addr = self.try_unwrap_stored(mem_addr)?;
        self.mem_read(mem_addr, i)
    }

    /// This is a safe variant of [`Store::mem_write`].
    pub fn mem_write_checked(
        &mut self,
        mem_addr: Stored<MemAddr>,
        i: u32,
        byte: u8,
    ) -> Result<(), RuntimeError> {
        let mem_addr = self.try_unwrap_stored(mem_addr)?;

        self.mem_write(mem_addr, i, byte)
    }

    /// This is a safe variant of [`Store::mem_size`].
    pub fn mem_size_checked(&self, mem_addr: Stored<MemAddr>) -> Result<u32, RuntimeError> {
        let mem_addr = self.try_unwrap_stored(mem_addr)?;
        Ok(self.mem_size(mem_addr))
    }

    /// This is a safe variant of [`Store::mem_grow`].
    pub fn mem_grow_checked(
        &mut self,
        mem_addr: Stored<MemAddr>,
        n: u32,
    ) -> Result<(), RuntimeError> {
        let mem_addr = self.try_unwrap_stored(mem_addr)?;
        self.mem_grow(mem_addr, n)
    }

    /// This is a safe variant of [`Store::global_alloc`].
    pub fn global_alloc_checked(
        &mut self,
        global_type: GlobalType,
        val: StoredValue,
    ) -> Result<Stored<GlobalAddr>, RuntimeError> {
        let val = self.try_unwrap_value(val)?;
        self.global_alloc(global_type, val)
            .map(|global_addr| self.wrap_stored(global_addr))
    }

    /// This is a safe variant of [`Store::global_type`].
    pub fn global_type_checked(
        &self,
        global_addr: Stored<GlobalAddr>,
    ) -> Result<GlobalType, RuntimeError> {
        let global_addr = self.try_unwrap_stored(global_addr)?;
        Ok(self.global_type(global_addr))
    }

    /// This is a safe variant of [`Store::global_read`].
    pub fn global_read_checked(
        &self,
        global_addr: Stored<GlobalAddr>,
    ) -> Result<StoredValue, RuntimeError> {
        let global_addr = self.try_unwrap_stored(global_addr)?;
        let value = self.global_read(global_addr);
        Ok(self.wrap_value(value))
    }

    /// This is a safe variant of [`Store::global_write`].
    pub fn global_write_checked(
        &mut self,
        global_addr: Stored<GlobalAddr>,
        val: StoredValue,
    ) -> Result<(), RuntimeError> {
        let global_addr = self.try_unwrap_stored(global_addr)?;
        let val = self.try_unwrap_value(val)?;
        self.global_write(global_addr, val)
    }

    /// This is a safe variant of [`Store::reregister_module`].
    pub fn reregister_module_checked(
        &mut self,
        module_addr: Stored<ModuleAddr>,
        name: &str,
    ) -> Result<(), RuntimeError> {
        let module_addr = self.try_unwrap_stored(module_addr)?;
        self.reregister_module(module_addr, name)
    }

    /// This is a safe variant of [`Store::create_resumable`].
    pub fn create_resumable_checked(
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
        let resumable_ref = self.create_resumable(func_addr, params, maybe_fuel)?;
        Ok(self.wrap_stored(resumable_ref))
    }

    /// This is a safe variant of [`Store::resume`].
    pub fn resume_checked(
        &mut self,
        resumable_ref: Stored<ResumableRef>,
    ) -> Result<StoredRunState, RuntimeError> {
        let resumable_ref = self.try_unwrap_stored(resumable_ref)?;
        let run_state = self.resume(resumable_ref)?;
        Ok(self.wrap_run_state(run_state))
    }

    /// This is a safe variant of [`Store::access_fuel_mut`].
    // TODO `&mut Stored<...>` seems off as a parameter type. Instead it should
    // be `Stored<ResumableRef>`
    pub fn access_fuel_mut_checked<R>(
        &mut self,
        resumable_ref: &mut Stored<ResumableRef>,
        f: impl FnOnce(&mut Option<u32>) -> R,
    ) -> Result<R, RuntimeError> {
        let resumable_ref = self.try_unwrap_stored(resumable_ref.as_mut())?;
        self.access_fuel_mut(resumable_ref, f)
    }

    /// This is a safe variant of [`Store::invoke_without_fuel`].
    pub fn invoke_without_fuel_checked(
        &mut self,
        func_addr: Stored<FuncAddr>,
        params: Vec<StoredValue>,
    ) -> Result<Vec<StoredValue>, RuntimeError> {
        let func_addr = self.try_unwrap_stored(func_addr)?;
        let params = params
            .into_iter()
            .map(|param| self.try_unwrap_value(param))
            .collect::<Result<Vec<Value>, RuntimeError>>()?;
        let returns = self.invoke_without_fuel(func_addr, params)?;
        let returns = returns
            .into_iter()
            .map(|return_value| self.wrap_value(return_value))
            .collect();
        Ok(returns)
    }
}

/// A generic stored wrapper. This is mostly used to wrap address types.
pub struct Stored<T> {
    id: StoreId,
    inner: T,
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

impl<T> Stored<T> {
    fn as_mut(&mut self) -> Stored<&mut T> {
        Stored {
            id: self.id,
            inner: &mut self.inner,
        }
    }
}

/// A stored variant of [`Value`]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StoredValue {
    I32(u32),
    I64(u64),
    F32(F32),
    F64(F64),
    V128([u8; 16]),
    Ref(StoredRef),
}

/// A stored variant of [`Ref`]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StoredRef {
    Null(RefType),
    Func(Stored<FuncAddr>),
    Extern(Stored<ExternAddr>),
}

/// A stored variant of [`ExternVal`]
#[derive(Debug, Copy, Clone)]
pub enum StoredExternVal {
    Func(Stored<FuncAddr>),
    Table(Stored<TableAddr>),
    Mem(Stored<MemAddr>),
    Global(Stored<GlobalAddr>),
}

/// A stored variant of [`RunState`]
pub enum StoredRunState {
    Finished(Vec<StoredValue>),
    Resumable {
        resumable_ref: Stored<ResumableRef>,
        required_fuel: NonZeroU32,
    },
}
