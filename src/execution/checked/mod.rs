//! Definitions for checked, safe variants of methods defined on [`Store`].
//!
//! All methods are simply wrappers which ensure that the addresses used belong to the current store.

use core::{
    num::NonZeroU32,
    sync::atomic::{AtomicU64, Ordering},
};

use crate::{
    addrs::{FuncAddr, GlobalAddr, MemAddr, ModuleAddr, TableAddr},
    config::Config,
    core::reader::types::{FuncType, MemType, TableType},
    linker::Linker,
    resumable::{ResumableRef, RunState},
    value::{Ref, ValueTypeMismatchError},
    ExternVal, GlobalType, InstantiationOutcome, RuntimeError, Store, ValidationInfo, Value,
};
use alloc::{string::String, vec::Vec};

mod interop;
mod value;

pub use interop::*;
pub use value::*;

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

impl<'b, T: Config> Store<'b, T> {
    // Note: `pub fn new_checked()` is missing, because it does not interact
    // with any stored objects.

    /// This is a safe variant of [`Store::module_instantiate_unchecked`].
    pub fn module_instantiate(
        &mut self,
        validation_info: &ValidationInfo<'b>,
        extern_vals: Vec<StoredExternVal>,
        maybe_fuel: Option<u32>,
    ) -> Result<StoredInstantiationOutcome, RuntimeError> {
        let extern_vals = extern_vals
            .into_iter()
            .map(|extern_val| self.id.try_unwrap_extern_val(extern_val))
            .collect::<Result<Vec<ExternVal>, RuntimeError>>()?;

        self.module_instantiate_unchecked(validation_info, extern_vals, maybe_fuel)
            .map(|instantiation_outcome| self.id.wrap_instantiation_outcome(instantiation_outcome))
    }

    /// This is a safe variant of [`Store::instance_export_unchecked`].
    pub fn instance_export(
        &self,
        module_addr: Stored<ModuleAddr>,
        name: &str,
    ) -> Result<StoredExternVal, RuntimeError> {
        let module_addr = self.id.try_unwrap_stored(module_addr)?;
        self.instance_export_unchecked(module_addr, name)
            .map(|extern_val| self.id.wrap_extern_val(extern_val))
    }

    // Note: `pub fn func_alloc(&mut self, ...)` is missing, because it would
    // require changes in the core interpreter.

    /// This is a safe variant of [`Store::func_type_unchecked`].
    pub fn func_type(&self, func_addr: Stored<FuncAddr>) -> Result<FuncType, RuntimeError> {
        let func_addr = self.id.try_unwrap_stored(func_addr)?;
        Ok(self.func_type_unchecked(func_addr))
    }

    /// This is a safe variant of [`Store::invoke_unchecked`].
    pub fn invoke(
        &mut self,
        func_addr: Stored<FuncAddr>,
        params: Vec<StoredValue>,
        maybe_fuel: Option<u32>,
    ) -> Result<StoredRunState, RuntimeError> {
        let func_addr = self.id.try_unwrap_stored(func_addr)?;
        let params = params
            .into_iter()
            .map(|value| self.id.try_unwrap_value(value))
            .collect::<Result<Vec<Value>, RuntimeError>>()?;

        self.invoke_unchecked(func_addr, params, maybe_fuel)
            .map(|run_state| self.id.wrap_run_state(run_state))
    }

    /// This is a safe variant of [`Store::table_alloc_unchecked`].
    pub fn table_alloc(
        &mut self,
        table_type: TableType,
        r#ref: StoredRef,
    ) -> Result<Stored<TableAddr>, RuntimeError> {
        let r#ref = self.id.try_unwrap_ref(r#ref)?;
        self.table_alloc_unchecked(table_type, r#ref)
            .map(|table_addr| self.id.wrap_stored(table_addr))
    }

    /// This is a safe variant of [`Store::table_type_unchecked`].
    pub fn table_type(&self, table_addr: Stored<TableAddr>) -> Result<TableType, RuntimeError> {
        let table_addr = self.id.try_unwrap_stored(table_addr)?;
        Ok(self.table_type_unchecked(table_addr))
    }

    /// This is a safe variant of [`Store::table_read_unchecked`].
    pub fn table_read(
        &self,
        table_addr: Stored<TableAddr>,
        i: u32,
    ) -> Result<StoredRef, RuntimeError> {
        let table_addr = self.id.try_unwrap_stored(table_addr)?;
        self.table_read_unchecked(table_addr, i)
            .map(|r#ref| self.id.wrap_ref(r#ref))
    }

    /// This is a safe variant of [`Store::table_write_unchecked`].
    pub fn table_write(
        &mut self,
        table_addr: Stored<TableAddr>,
        i: u32,
        r#ref: StoredRef,
    ) -> Result<(), RuntimeError> {
        let table_addr = self.id.try_unwrap_stored(table_addr)?;
        let r#ref = self.id.try_unwrap_ref(r#ref)?;
        self.table_write_unchecked(table_addr, i, r#ref)
    }

    /// This is a safe variant of [`Store::table_size_unchecked`].
    pub fn table_size(&self, table_addr: Stored<TableAddr>) -> Result<u32, RuntimeError> {
        let table_addr = self.id.try_unwrap_stored(table_addr)?;

        Ok(self.table_size_unchecked(table_addr))
    }

    /// This is a safe variant of [`Store::mem_alloc_unchecked`].
    pub fn mem_alloc(&mut self, mem_type: MemType) -> Stored<MemAddr> {
        let mem_addr = self.mem_alloc_unchecked(mem_type);
        self.id.wrap_stored(mem_addr)
    }

    /// This is a safe variant of [`Store::mem_type_unchecked`].
    pub fn mem_type(&self, mem_addr: Stored<MemAddr>) -> Result<MemType, RuntimeError> {
        let mem_addr = self.id.try_unwrap_stored(mem_addr)?;
        Ok(self.mem_type_unchecked(mem_addr))
    }

    /// This is a safe variant of [`Store::mem_read_unchecked`].
    pub fn mem_read(&self, mem_addr: Stored<MemAddr>, i: u32) -> Result<u8, RuntimeError> {
        let mem_addr = self.id.try_unwrap_stored(mem_addr)?;
        self.mem_read_unchecked(mem_addr, i)
    }

    /// This is a safe variant of [`Store::mem_write_unchecked`].
    pub fn mem_write(
        &mut self,
        mem_addr: Stored<MemAddr>,
        i: u32,
        byte: u8,
    ) -> Result<(), RuntimeError> {
        let mem_addr = self.id.try_unwrap_stored(mem_addr)?;

        self.mem_write_unchecked(mem_addr, i, byte)
    }

    /// This is a safe variant of [`Store::mem_size_unchecked`].
    pub fn mem_size(&self, mem_addr: Stored<MemAddr>) -> Result<u32, RuntimeError> {
        let mem_addr = self.id.try_unwrap_stored(mem_addr)?;
        Ok(self.mem_size_unchecked(mem_addr))
    }

    /// This is a safe variant of [`Store::mem_grow_unchecked`].
    pub fn mem_grow(&mut self, mem_addr: Stored<MemAddr>, n: u32) -> Result<(), RuntimeError> {
        let mem_addr = self.id.try_unwrap_stored(mem_addr)?;
        self.mem_grow_unchecked(mem_addr, n)
    }

    /// This is a safe variant of [`Store::global_alloc_unchecked`].
    pub fn global_alloc(
        &mut self,
        global_type: GlobalType,
        val: StoredValue,
    ) -> Result<Stored<GlobalAddr>, RuntimeError> {
        let val = self.id.try_unwrap_value(val)?;
        self.global_alloc_unchecked(global_type, val)
            .map(|global_addr| self.id.wrap_stored(global_addr))
    }

    /// This is a safe variant of [`Store::global_type_unchecked`].
    pub fn global_type(&self, global_addr: Stored<GlobalAddr>) -> Result<GlobalType, RuntimeError> {
        let global_addr = self.id.try_unwrap_stored(global_addr)?;
        Ok(self.global_type_unchecked(global_addr))
    }

    /// This is a safe variant of [`Store::global_read_unchecked`].
    pub fn global_read(
        &self,
        global_addr: Stored<GlobalAddr>,
    ) -> Result<StoredValue, RuntimeError> {
        let global_addr = self.id.try_unwrap_stored(global_addr)?;
        let value = self.global_read_unchecked(global_addr);
        Ok(self.id.wrap_value(value))
    }

    /// This is a safe variant of [`Store::global_write_unchecked`].
    pub fn global_write(
        &mut self,
        global_addr: Stored<GlobalAddr>,
        val: StoredValue,
    ) -> Result<(), RuntimeError> {
        let global_addr = self.id.try_unwrap_stored(global_addr)?;
        let val = self.id.try_unwrap_value(val)?;
        self.global_write_unchecked(global_addr, val)
    }

    /// This is a safe variant of [`Store::create_resumable_unchecked`].
    pub fn create_resumable(
        &self,
        func_addr: Stored<FuncAddr>,
        params: Vec<StoredValue>,
        maybe_fuel: Option<u32>,
    ) -> Result<Stored<ResumableRef>, RuntimeError> {
        let func_addr = self.id.try_unwrap_stored(func_addr)?;
        let params = params
            .into_iter()
            .map(|param| self.id.try_unwrap_value(param))
            .collect::<Result<Vec<Value>, RuntimeError>>()?;
        let resumable_ref = self.create_resumable_unchecked(func_addr, params, maybe_fuel)?;
        Ok(self.id.wrap_stored(resumable_ref))
    }

    /// This is a safe variant of [`Store::resume_unchecked`].
    pub fn resume(
        &mut self,
        resumable_ref: Stored<ResumableRef>,
    ) -> Result<StoredRunState, RuntimeError> {
        let resumable_ref = self.id.try_unwrap_stored(resumable_ref)?;
        let run_state = self.resume_unchecked(resumable_ref)?;
        Ok(self.id.wrap_run_state(run_state))
    }

    /// This is a safe variant of [`Store::access_fuel_mut_unchecked`].
    // TODO `&mut Stored<...>` seems off as a parameter type. Instead it should
    // be `Stored<ResumableRef>`
    pub fn access_fuel_mut<R>(
        &mut self,
        resumable_ref: &mut Stored<ResumableRef>,
        f: impl FnOnce(&mut Option<u32>) -> R,
    ) -> Result<R, RuntimeError> {
        let resumable_ref = self.id.try_unwrap_stored(resumable_ref.as_mut())?;
        self.access_fuel_mut_unchecked(resumable_ref, f)
    }

    // Note: `pub fn func_alloc_typed(&mut self, ...)` is missing, because of
    // the same reason `func_alloc` is missing.

    /// This is a safe variant of [`Store::invoke_without_fuel_unchecked`].
    pub fn invoke_without_fuel(
        &mut self,
        func_addr: Stored<FuncAddr>,
        params: Vec<StoredValue>,
    ) -> Result<Vec<StoredValue>, RuntimeError> {
        let func_addr = self.id.try_unwrap_stored(func_addr)?;
        let params = params
            .into_iter()
            .map(|param| self.id.try_unwrap_value(param))
            .collect::<Result<Vec<Value>, RuntimeError>>()?;
        let returns = self.invoke_without_fuel_unchecked(func_addr, params)?;
        let returns = returns
            .into_iter()
            .map(|return_value| self.id.wrap_value(return_value))
            .collect();
        Ok(returns)
    }

    /// This is a safe variant of [`Store::invoke_typed_without_fuel_unchecked`].
    pub fn invoke_typed_without_fuel<
        Params: StoredInteropValueList,
        Returns: StoredInteropValueList,
    >(
        &mut self,
        function: Stored<FuncAddr>,
        params: Params,
    ) -> Result<Returns, RuntimeError> {
        self.invoke_without_fuel(function, params.into_values())
            .and_then(|results| {
                Returns::try_from_values(results.into_iter()).map_err(|ValueTypeMismatchError| {
                    RuntimeError::FunctionInvocationSignatureMismatch
                })
            })
    }
}

impl Linker {
    /// This is a safe variant of [`Linker::define_unchecked`].
    pub fn define(
        &mut self,
        module_name: String,
        name: String,
        extern_val: StoredExternVal,
    ) -> Result<(), RuntimeError> {
        let linker_store_id = *self.store_id.get_or_insert(extern_val.id());
        if linker_store_id != extern_val.id() {
            return Err(RuntimeError::StoreIdMismatch);
        }

        let extern_val = linker_store_id.try_unwrap_extern_val(extern_val)?;

        self.define_unchecked(module_name, name, extern_val)
    }

    /// This is a safe variant of [`Linker::define_module_instance_unchecked`].
    pub fn define_module_instance<T: Config>(
        &mut self,
        store: &Store<T>,
        module_name: String,
        module: Stored<ModuleAddr>,
    ) -> Result<(), RuntimeError> {
        let linker_store_id = *self.store_id.get_or_insert(module.id());
        if linker_store_id != module.id() {
            return Err(RuntimeError::StoreIdMismatch);
        }

        let module = linker_store_id.try_unwrap_stored(module)?;

        self.define_module_instance_unchecked(store, module_name, module)
    }

    /// This is a safe variant of [`Linker::get_unchecked`].
    ///
    /// # Interaction with unchecked API
    ///
    /// This method is able to find externs defined through the unchecked
    /// `define` methods.  However, for this to work, at least one of the
    /// following methods must have been called successfully:
    /// [`Linker::define`], [`Linker::define_module_instance`],
    /// [`Linker::module_instantiate`]. Otherwise, this method may spuriously
    /// return an error.
    ///
    /// Therefore, it is advised to commit to either the unchecked or the
    /// checked API for each individual [`Linker`].
    ///
    /// # Errors
    ///
    /// - [`RuntimeError::LinkerNotYetAssociatedWithStoreId`]
    /// - [`RuntimeError::UnableToResolveExternLookup`]
    pub fn get(&self, module_name: String, name: String) -> Result<StoredExternVal, RuntimeError> {
        let Some(linker_store_id) = self.store_id else {
            // We know that nothing was defined in this linker context through
            // the checked methods yet, because `self.store_id` has not been set
            // yet. Therefore, a get would always return `None`.

            // However, when an unchecked `define` method was used before, we
            // also have to return `None` here, because even if the lookup for
            // `module_name` and `name` returns something, we cannot attach a
            // store id to it.

            return Err(RuntimeError::LinkerNotYetAssociatedWithStoreId);
        };

        let extern_val = self
            .get_unchecked(module_name, name)
            .ok_or(RuntimeError::UnableToResolveExternLookup)?;

        let stored_extern_val = linker_store_id.wrap_extern_val(extern_val);
        Ok(stored_extern_val)
    }

    /// This is a safe variant of [`Linker::instantiate_pre_unchecked`].
    ///
    /// # Interaction with unchecked API
    ///
    /// See [`Linker::get`]
    ///
    /// # Errors
    ///
    /// - [`RuntimeError::LinkerNotYetAssociatedWithStoreId`]
    /// - [`RuntimeError::UnableToResolveExternLookup`]
    pub fn instantiate_pre(
        &self,
        validation_info: &ValidationInfo,
    ) -> Result<Vec<StoredExternVal>, RuntimeError> {
        // Special case: If the module has no imports, we don't perform any
        // linking. We need this special case, so that a `Linker`, that has not
        // yet been associated with some `Store`, can still be used to
        // pre-instantiate modules.
        if validation_info.imports.is_empty() {
            return Ok(Vec::new());
        }

        let Some(linker_store_id) = self.store_id else {
            // We are not able to perform safe linking (see this method's and
            // `Linker::get`'s documentations).
            return Err(RuntimeError::LinkerNotYetAssociatedWithStoreId);
        };

        let extern_vals = self.instantiate_pre_unchecked(validation_info)?;

        let stored_extern_vals = extern_vals
            .into_iter()
            .map(|extern_val| linker_store_id.wrap_extern_val(extern_val))
            .collect();

        Ok(stored_extern_vals)
    }

    /// This is a safe variant of [`Linker::module_instantiate_unchecked`].
    pub fn module_instantiate<'b, T: Config>(
        &mut self,
        store: &mut Store<'b, T>,
        validation_info: &ValidationInfo<'b>,
        maybe_fuel: Option<u32>,
    ) -> Result<StoredInstantiationOutcome, RuntimeError> {
        let linker_store_id = *self.store_id.get_or_insert(store.id);
        if linker_store_id != store.id {
            return Err(RuntimeError::StoreIdMismatch);
        }

        let instantiation_outcome =
            self.module_instantiate_unchecked(store, validation_info, maybe_fuel)?;
        let stored_instantiation_outcome =
            linker_store_id.wrap_instantiation_outcome(instantiation_outcome);

        Ok(stored_instantiation_outcome)
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

    fn id(&self) -> StoreId {
        self.id
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

impl StoredExternVal {
    fn id(self) -> StoreId {
        match self {
            StoredExternVal::Func(stored) => stored.id(),
            StoredExternVal::Table(stored) => stored.id(),
            StoredExternVal::Mem(stored) => stored.id(),
            StoredExternVal::Global(stored) => stored.id(),
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
pub enum StoredRunState {
    Finished {
        values: Vec<StoredValue>,
        maybe_remaining_fuel: Option<u32>,
    },
    Resumable {
        resumable_ref: Stored<ResumableRef>,
        required_fuel: NonZeroU32,
    },
}

/// A stored variant of [`InstantiationOutcome`]
pub struct StoredInstantiationOutcome {
    pub module_addr: Stored<ModuleAddr>,
    pub maybe_remaining_fuel: Option<u32>,
}

impl StoreId {
    /// Associates some value of type `U` with this store id, producing a
    /// [`Stored<U>`] object. This object can then be unwrapped later.
    ///
    /// See also: [`StoreId::try_unwrap_stored`].
    fn wrap_stored<U>(self, inner: U) -> Stored<U> {
        Stored { id: self, inner }
    }

    /// Validates that some [`Stored<U>`] object has this store id.  If this is
    /// true, the inner value of type `U` is returned, otherwise an error is
    /// returned.
    ///
    /// See also: [`StoreId::wrap_stored`].
    ///
    /// # Errors
    /// - [`RuntimeError::StoreIdMismatch`]
    pub(crate) fn try_unwrap_stored<U>(self, stored: Stored<U>) -> Result<U, RuntimeError> {
        if self == stored.id {
            Ok(stored.inner)
        } else {
            Err(RuntimeError::StoreIdMismatch)
        }
    }

    /// Associates some [`Value`] with this store id, producing a
    /// [`StoredValue`] object. This object can be matched against or unwrapped
    /// later.
    ///
    /// See also: [`StoreId::try_unwrap_value`].
    pub(crate) fn wrap_value(self, value: Value) -> StoredValue {
        match value {
            Value::I32(x) => StoredValue::I32(x),
            Value::I64(x) => StoredValue::I64(x),
            Value::F32(x) => StoredValue::F32(x),
            Value::F64(x) => StoredValue::F64(x),
            Value::V128(x) => StoredValue::V128(x),
            Value::Ref(r#ref) => StoredValue::Ref(self.wrap_ref(r#ref)),
        }
    }

    /// Validates that some [`StoredValue`] has a this store id. If this is
    /// true, the value is returned as a [`Value`], otherwise an error is
    /// returned.
    ///
    /// See also: [`StoreId::wrap_value`].
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

    /// Associates some [`Ref`] with this store id, producing a [`StoredRef`]
    /// object. This object can be matched against or unwrapped later.
    ///
    /// See also: [`StoreId::try_unwrap_ref`].
    pub(crate) fn wrap_ref(&self, r#ref: Ref) -> StoredRef {
        match r#ref {
            Ref::Null(ref_type) => StoredRef::Null(ref_type),
            Ref::Func(func_addr) => StoredRef::Func(self.wrap_stored(func_addr)),
            Ref::Extern(extern_addr) => StoredRef::Extern(extern_addr),
        }
    }

    /// Validates that some [`StoredRef`] has this store id. If this is true,
    /// the value is returned as a [`Ref`], otherwise an error is returned.
    ///
    /// See also: [`StoreId::wrap_ref`].
    ///
    /// # Errors
    /// - [`RuntimeError::StoreIdMismatch`]
    pub(crate) fn try_unwrap_ref(&self, stored_ref: StoredRef) -> Result<Ref, RuntimeError> {
        let r#ref = match stored_ref {
            StoredRef::Null(ref_type) => Ref::Null(ref_type),
            StoredRef::Func(func_addr) => Ref::Func(self.try_unwrap_stored(func_addr)?),
            StoredRef::Extern(extern_addr) => Ref::Extern(extern_addr),
        };
        Ok(r#ref)
    }

    /// Associates some [`ExternVal`] with this store id, producing a
    /// [`StoredExternVal`] object. This object can only be matched against to
    /// obtain different kinds of [`Stored<...>`] objects.
    ///
    /// See also: [`StoreId::try_unwrap_stored`].
    pub(crate) fn wrap_extern_val(&self, extern_val: ExternVal) -> StoredExternVal {
        match extern_val {
            ExternVal::Func(addr) => StoredExternVal::Func(self.wrap_stored(addr)),
            ExternVal::Table(addr) => StoredExternVal::Table(self.wrap_stored(addr)),
            ExternVal::Mem(addr) => StoredExternVal::Mem(self.wrap_stored(addr)),
            ExternVal::Global(addr) => StoredExternVal::Global(self.wrap_stored(addr)),
        }
    }

    /// Validates that some [`StoredExternVal`] has this store id. If this is
    /// true, the value is returned as a [`ExternVal`], otherwise an error is
    /// returned.
    ///
    /// See also: [`StoreId::wrap_extern_val`]
    ///
    /// # Errors
    /// - [`RuntimeError::StoreIdMismatch`]
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

    /// Associates some [`ExternVal`] with this store id, producing a
    /// [`StoredExternVal`] object. This object can only be matched against to
    /// obtain [`Stored<...>`] or [`StoredValue`] objects.
    ///
    /// See also: [`StoreId::try_unwrap_stored`], [`StoreId::try_unwrap_value`].
    pub(crate) fn wrap_run_state(&self, run_state: RunState) -> StoredRunState {
        match run_state {
            RunState::Finished {
                values,
                maybe_remaining_fuel,
            } => StoredRunState::Finished {
                values: values
                    .into_iter()
                    .map(|value| self.wrap_value(value))
                    .collect(),
                maybe_remaining_fuel,
            },
            RunState::Resumable {
                resumable_ref,
                required_fuel,
            } => StoredRunState::Resumable {
                resumable_ref: self.wrap_stored(resumable_ref),
                required_fuel,
            },
        }
    }

    /// Associates some [`InstantiationOutcome`] with this store id, producing a
    /// [`StoredInstantiationOutcome`] object. This object can only be matched
    /// against to obtain [`Stored<...>`] values.
    ///
    /// See also: [`StoreId::try_unwrap_stored`], [`StoreId::try_unwrap_value`].
    pub(crate) fn wrap_instantiation_outcome(
        &self,
        instantiation_outcome: InstantiationOutcome,
    ) -> StoredInstantiationOutcome {
        StoredInstantiationOutcome {
            module_addr: self.wrap_stored(instantiation_outcome.module_addr),
            maybe_remaining_fuel: instantiation_outcome.maybe_remaining_fuel,
        }
    }
}
