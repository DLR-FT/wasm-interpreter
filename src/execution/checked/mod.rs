//! Definitions for checked, safe variants of methods defined on [`Store`] and
//! [`Linker`].
//!
//! This module defines extensions in the form of new types and new methods. It
//! only relies on the fact that the [`Store`] and the [`Linker`] both store a
//! [`StoreId`]. No other changes are required to be made to the main
//! interpreter for this checked API.
//!
//!
//! All extension methods defined in this module use special _stored_ objects.
//! These objects are essentially normal objects like [`FuncAddr`], [`RunState`]
//! or [`Value`](crate::execution::Value). However, they also contain an
//! additional field of type [`StoreId`] as a tag to know to which [`Store`]
//! they belong to.
//!
//! While this is easy for address types like [`FuncAddr`] or [`MemAddr`], some
//! types are enums and their variants are visible to the user. For example,
//! consider the [`Value`](crate::execution::Value) enum, where users have full
//! access to all of its variants. To be able to attach a tag only to the
//! [`Value::Ref`](crate::execution::Value::Ref) variant of this enum, the
//! entire enum has to be re-defined. The result is a completely new type
//! [`StoredValue`].

use core::num::NonZeroU32;

use crate::{
    addrs::{FuncAddr, GlobalAddr, MemAddr, ModuleAddr, TableAddr},
    config::Config,
    core::reader::types::{FuncType, MemType, TableType},
    linker::Linker,
    resumable::{ResumableRef, RunState},
    ExternVal, GlobalType, InstantiationOutcome, RuntimeError, Store, StoreId, ValidationInfo,
};
use alloc::{string::String, vec::Vec};

mod interop;
mod value;

pub use interop::*;
pub use value::*;

// All functions in this impl block must occur in the same order as they are
// defined in for the unchecked `Store` methods. Also all functions must follow
// the same implementation scheme to make sure they are only light wrappers:
//
// 1. try unwrap [stored parameter objects]
// 2. call [unchecked method]
// 3. rewrap [results into stored objects]
// 4. return [stored result objects]
impl<'b, T: Config> Store<'b, T> {
    // `fn new_checked` is missing, because it does not interact with any stored
    // objects.

    /// This is a safe variant of [`Store::module_instantiate_unchecked`].
    pub fn module_instantiate(
        &mut self,
        validation_info: &ValidationInfo<'b>,
        extern_vals: Vec<StoredExternVal>,
        maybe_fuel: Option<u32>,
    ) -> Result<StoredInstantiationOutcome, RuntimeError> {
        // 1. try unwrap
        let extern_vals = extern_vals
            .into_iter()
            .map(|extern_val| extern_val.try_unwrap_into_bare(self.id))
            .collect::<Result<Vec<ExternVal>, RuntimeError>>()?;
        // 2. call
        let instantiation_outcome =
            self.module_instantiate_unchecked(validation_info, extern_vals, maybe_fuel)?;
        // 3. rewrap
        // Safety: The `InstantiationOutcome` just came from the current store.
        let stored_instantiation_outcome =
            unsafe { StoredInstantiationOutcome::from_bare(instantiation_outcome, self.id) };
        // 4. return
        Ok(stored_instantiation_outcome)
    }

    /// This is a safe variant of [`Store::instance_export_unchecked`].
    pub fn instance_export(
        &self,
        module_addr: Stored<ModuleAddr>,
        name: &str,
    ) -> Result<StoredExternVal, RuntimeError> {
        // 1. try unwrap
        let module_addr = module_addr.try_unwrap_into_bare(self.id)?;
        // 2. call
        let extern_val = self.instance_export_unchecked(module_addr, name)?;
        // 3. rewrap
        // Safety: The `ExternVal` just came from the current store.
        let stored_extern_val = unsafe { StoredExternVal::from_bare(extern_val, self.id) };
        // 4. return
        Ok(stored_extern_val)
    }

    // `fn func_alloc` is missing, because it would require changes in the core
    // interpreter.

    /// This is a safe variant of [`Store::func_type_unchecked`].
    pub fn func_type(&self, func_addr: Stored<FuncAddr>) -> Result<FuncType, RuntimeError> {
        // 1. try unwrap
        let func_addr = func_addr.try_unwrap_into_bare(self.id)?;
        // 2. call
        let func_type = self.func_type_unchecked(func_addr);
        // 3. rewrap
        // `FuncType` does not have a stored variant.
        // 4. return
        Ok(func_type)
    }

    /// This is a safe variant of [`Store::invoke_unchecked`].
    pub fn invoke(
        &mut self,
        func_addr: Stored<FuncAddr>,
        params: Vec<StoredValue>,
        maybe_fuel: Option<u32>,
    ) -> Result<StoredRunState, RuntimeError> {
        // 1. try unwrap
        let func_addr = func_addr.try_unwrap_into_bare(self.id)?;
        let params = try_unwrap_values(params, self.id)?;
        // 2. call
        let run_state = self.invoke_unchecked(func_addr, params, maybe_fuel)?;
        // 3. rewrap
        // Safety: The `RunState` just came from the current store.
        let stored_run_state = unsafe { StoredRunState::from_bare(run_state, self.id) };
        // 4. return
        Ok(stored_run_state)
    }

    /// This is a safe variant of [`Store::table_alloc_unchecked`].
    pub fn table_alloc(
        &mut self,
        table_type: TableType,
        r#ref: StoredRef,
    ) -> Result<Stored<TableAddr>, RuntimeError> {
        // 1. try unwrap
        let r#ref = r#ref.try_unwrap_into_bare(self.id)?;
        // 2. call
        let table_addr = self.table_alloc_unchecked(table_type, r#ref)?;
        // 3. rewrap
        // Safety: The `TableAddr` just came from the current store.
        let stored_table_addr = unsafe { Stored::from_bare(table_addr, self.id) };
        // 4. return
        Ok(stored_table_addr)
    }

    /// This is a safe variant of [`Store::table_type_unchecked`].
    pub fn table_type(&self, table_addr: Stored<TableAddr>) -> Result<TableType, RuntimeError> {
        // 1. try unwrap
        let table_addr = table_addr.try_unwrap_into_bare(self.id)?;
        // 2. call
        let table_type = self.table_type_unchecked(table_addr);
        // 3. rewrap
        // `TableType` has no stored variant.
        // 4. return
        Ok(table_type)
    }

    /// This is a safe variant of [`Store::table_read_unchecked`].
    pub fn table_read(
        &self,
        table_addr: Stored<TableAddr>,
        i: u32,
    ) -> Result<StoredRef, RuntimeError> {
        // 1. try unwrap
        let table_addr = table_addr.try_unwrap_into_bare(self.id)?;
        // 2. call
        let r#ref = self.table_read_unchecked(table_addr, i)?;
        // 3. rewrap
        // Safety: The `Ref` ust came from the current store.
        let stored_ref = unsafe { StoredRef::from_bare(r#ref, self.id) };
        // 4. return
        Ok(stored_ref)
    }

    /// This is a safe variant of [`Store::table_write_unchecked`].
    pub fn table_write(
        &mut self,
        table_addr: Stored<TableAddr>,
        i: u32,
        r#ref: StoredRef,
    ) -> Result<(), RuntimeError> {
        // 1. try unwrap
        let table_addr = table_addr.try_unwrap_into_bare(self.id)?;
        let r#ref = r#ref.try_unwrap_into_bare(self.id)?;
        // 2. call
        self.table_write_unchecked(table_addr, i, r#ref)?;
        // 3. rewrap
        // result is the unit type.
        // 4. return
        Ok(())
    }

    /// This is a safe variant of [`Store::table_size_unchecked`].
    pub fn table_size(&self, table_addr: Stored<TableAddr>) -> Result<u32, RuntimeError> {
        // 1. try unwrap
        let table_addr = table_addr.try_unwrap_into_bare(self.id)?;
        // 2. call
        let table_size = self.table_size_unchecked(table_addr);
        // 3. rewrap
        // table size has no stored variant.
        // 4. return
        Ok(table_size)
    }

    /// This is a safe variant of [`Store::mem_alloc_unchecked`].
    #[allow(clippy::let_and_return)] // reason = "to follow the 1234 structure"
    pub fn mem_alloc(&mut self, mem_type: MemType) -> Stored<MemAddr> {
        // 1. try unwrap
        // no stored parameters
        // 2. call
        let mem_addr = self.mem_alloc_unchecked(mem_type);
        // 3. rewrap
        // Safety: The `MemAddr` just came from the current store.
        let stored_mem_addr = unsafe { Stored::from_bare(mem_addr, self.id) };
        // 4. return
        stored_mem_addr
    }

    /// This is a safe variant of [`Store::mem_type_unchecked`].
    pub fn mem_type(&self, mem_addr: Stored<MemAddr>) -> Result<MemType, RuntimeError> {
        // 1. try unwrap
        let mem_addr = mem_addr.try_unwrap_into_bare(self.id)?;
        // 2. call
        let mem_type = self.mem_type_unchecked(mem_addr);
        // 3. rewrap
        // `MemType` does not have a stored variant.
        // 4. return
        Ok(mem_type)
    }

    /// This is a safe variant of [`Store::mem_read_unchecked`].
    pub fn mem_read(&self, mem_addr: Stored<MemAddr>, i: u32) -> Result<u8, RuntimeError> {
        // 1. try unwrap
        let mem_addr = mem_addr.try_unwrap_into_bare(self.id)?;
        // 2. call
        let byte = self.mem_read_unchecked(mem_addr, i)?;
        // 3. rewrap
        // a single byte does not have a stored variant.
        // 4. return
        Ok(byte)
    }

    /// This is a safe variant of [`Store::mem_write_unchecked`].
    pub fn mem_write(
        &mut self,
        mem_addr: Stored<MemAddr>,
        i: u32,
        byte: u8,
    ) -> Result<(), RuntimeError> {
        // 1. try unwrap
        let mem_addr = mem_addr.try_unwrap_into_bare(self.id)?;
        // 2. call
        self.mem_write_unchecked(mem_addr, i, byte)?;
        // 3. rewrap
        // result is the unit type.
        // 4. return
        Ok(())
    }

    /// This is a safe variant of [`Store::mem_size_unchecked`].
    pub fn mem_size(&self, mem_addr: Stored<MemAddr>) -> Result<u32, RuntimeError> {
        // 1. try unwrap
        let mem_addr = mem_addr.try_unwrap_into_bare(self.id)?;
        // 2. call
        let mem_size = self.mem_size_unchecked(mem_addr);
        // 3. rewrap
        // mem size does not have a stored variant.
        // 4. return
        Ok(mem_size)
    }

    /// This is a safe variant of [`Store::mem_grow_unchecked`].
    pub fn mem_grow(&mut self, mem_addr: Stored<MemAddr>, n: u32) -> Result<(), RuntimeError> {
        // 1. try unwrap
        let mem_addr = mem_addr.try_unwrap_into_bare(self.id)?;
        // 2. call
        self.mem_grow_unchecked(mem_addr, n)?;
        // 3. rewrap
        // result is the unit type.
        // 4. return
        Ok(())
    }

    /// This is a safe variant of [`Store::global_alloc_unchecked`].
    pub fn global_alloc(
        &mut self,
        global_type: GlobalType,
        val: StoredValue,
    ) -> Result<Stored<GlobalAddr>, RuntimeError> {
        // 1. try unwrap
        let val = val.try_unwrap_into_bare(self.id)?;
        // 2. call
        let global_addr = self.global_alloc_unchecked(global_type, val)?;
        // 3. rewrap
        // Safety: The `GlobalAddr` just came from the current store.
        let stored_global_addr = unsafe { Stored::from_bare(global_addr, self.id) };
        // 4. return
        Ok(stored_global_addr)
    }

    /// This is a safe variant of [`Store::global_type_unchecked`].
    pub fn global_type(&self, global_addr: Stored<GlobalAddr>) -> Result<GlobalType, RuntimeError> {
        // 1. try unwrap
        let global_addr = global_addr.try_unwrap_into_bare(self.id)?;
        // 2. call
        let global_type = self.global_type_unchecked(global_addr);
        // 3. rewrap
        // `GlobalType` does not have a stored variant.
        // 4. return
        Ok(global_type)
    }

    /// This is a safe variant of [`Store::global_read_unchecked`].
    pub fn global_read(
        &self,
        global_addr: Stored<GlobalAddr>,
    ) -> Result<StoredValue, RuntimeError> {
        // 1. try unwrap
        let global_addr = global_addr.try_unwrap_into_bare(self.id)?;
        // 2. call
        let value = self.global_read_unchecked(global_addr);
        // 3. rewrap
        // Safety: The `Value` just came from the current store.
        let stored_value = unsafe { StoredValue::from_bare(value, self.id) };
        // 4. return
        Ok(stored_value)
    }

    /// This is a safe variant of [`Store::global_write_unchecked`].
    pub fn global_write(
        &mut self,
        global_addr: Stored<GlobalAddr>,
        val: StoredValue,
    ) -> Result<(), RuntimeError> {
        // 1. try unwrap
        let global_addr = global_addr.try_unwrap_into_bare(self.id)?;
        let val = val.try_unwrap_into_bare(self.id)?;
        // 2. call
        self.global_write_unchecked(global_addr, val)?;
        // 3. rewrap
        // result is the unit type.
        // 4. return
        Ok(())
    }

    /// This is a safe variant of [`Store::create_resumable_unchecked`].
    pub fn create_resumable(
        &self,
        func_addr: Stored<FuncAddr>,
        params: Vec<StoredValue>,
        maybe_fuel: Option<u32>,
    ) -> Result<Stored<ResumableRef>, RuntimeError> {
        // 1. try unwrap
        let func_addr = func_addr.try_unwrap_into_bare(self.id)?;
        let params = try_unwrap_values(params, self.id)?;
        // 2. call
        let resumable_ref = self.create_resumable_unchecked(func_addr, params, maybe_fuel)?;
        // 3. rewrap
        // Safety: The `ResumableRef` just came from the current store.
        let stored_resumable_ref = unsafe { Stored::from_bare(resumable_ref, self.id) };
        // 4. return
        Ok(stored_resumable_ref)
    }

    /// This is a safe variant of [`Store::resume_unchecked`].
    pub fn resume(
        &mut self,
        resumable_ref: Stored<ResumableRef>,
    ) -> Result<StoredRunState, RuntimeError> {
        // 1. try unwrap
        let resumable_ref = resumable_ref.try_unwrap_into_bare(self.id)?;
        // 2. call
        let run_state = self.resume_unchecked(resumable_ref)?;
        // 3. rewrap
        // Safety: The `RunState` just came from the current store.
        let stored_run_state = unsafe { StoredRunState::from_bare(run_state, self.id) };
        // 4. return
        Ok(stored_run_state)
    }

    /// This is a safe variant of [`Store::access_fuel_mut_unchecked`].
    // TODO `&mut Stored<...>` seems off as a parameter type. Instead it should
    // be `Stored<ResumableRef>`
    pub fn access_fuel_mut<R>(
        &mut self,
        resumable_ref: &mut Stored<ResumableRef>,
        f: impl FnOnce(&mut Option<u32>) -> R,
    ) -> Result<R, RuntimeError> {
        // 1. try unwrap
        let resumable_ref = resumable_ref.as_mut().try_unwrap_into_bare(self.id)?;
        // 2. call
        let r = self.access_fuel_mut_unchecked(resumable_ref, f)?;
        // 3. rewrap
        // result type `R` is generic.
        // 4. return
        Ok(r)
    }

    // `fn func_alloc_typed` is missing because of the same reason why `fn
    // func_alloc` is missing.

    /// This is a safe variant of [`Store::invoke_without_fuel_unchecked`].
    pub fn invoke_without_fuel(
        &mut self,
        func_addr: Stored<FuncAddr>,
        params: Vec<StoredValue>,
    ) -> Result<Vec<StoredValue>, RuntimeError> {
        // 1 try unwrap
        let func_addr = func_addr.try_unwrap_into_bare(self.id)?;
        let params = try_unwrap_values(params, self.id)?;
        // 2. call
        let returns = self.invoke_without_fuel_unchecked(func_addr, params)?;
        // 3. rewrap
        // Safety: All `Value`s just came from the current store.
        let returns = unsafe { wrap_vec_elements(returns, self.id) };
        // 4. return
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
        // 1. try unwrap
        let function = function.try_unwrap_into_bare(self.id)?;
        let params = try_unwrap_values(params.into_values(), self.id)?;
        // 2. call
        let returns = self.invoke_without_fuel_unchecked(function, params)?;
        // 3. rewrap
        // Safety: All `Value`s just came from the current store.
        let stored_returns = unsafe { wrap_vec_elements(returns, self.id) };
        // 4. return
        let stored_returns = Returns::try_from_values(stored_returns.into_iter())
            .map_err(|_| RuntimeError::FunctionInvocationSignatureMismatch)?;
        Ok(stored_returns)
    }

    /// This is a safe variant of [`Store::mem_access_mut_slice`].
    pub fn mem_access_mut_slice<R>(
        &self,
        memory: Stored<MemAddr>,
        accessor: impl FnOnce(&mut [u8]) -> R,
    ) -> Result<R, RuntimeError> {
        // 1. try unwrap
        let memory = memory.try_unwrap_into_bare(self.id)?;
        // 2. call
        let returns = self.mem_access_mut_slice_unchecked(memory, accessor);
        // 3. rewrap
        // result is generic
        // 4. return
        Ok(returns)
    }
}

// All functions in this impl block must occur in the same order as they are
// defined in for the unchecked [`Linker`] methods. Also all functions must
// follow the same implementation scheme to make sure they are only light
// wrappers:
//
// 1. get or insert the `StoreId` [of the store associated with the current `Linker`]
// 2. try unwrap [stored parameter objects]
// 3. call [unchecked method]
// 4. rewrap [results into stored objects]
// 5. return [stored result objects]
impl Linker {
    /// This is a safe variant of [`Linker::define_unchecked`].
    pub fn define(
        &mut self,
        module_name: String,
        name: String,
        extern_val: StoredExternVal,
    ) -> Result<(), RuntimeError> {
        // 1. get or insert the `StoreId`
        let extern_val_store_id = extern_val
            .id()
            .expect("this type to always contain a StoreId");
        let linker_store_id = *self.store_id.get_or_insert(extern_val_store_id);
        if linker_store_id != extern_val_store_id {
            return Err(RuntimeError::StoreIdMismatch);
        }
        // 2. try unwrap
        let extern_val = extern_val.try_unwrap_into_bare(linker_store_id)?;
        // 3. call
        self.define_unchecked(module_name, name, extern_val)?;
        // 4. rewrap
        // result is the unit type.
        // 5. return
        Ok(())
    }

    /// This is a safe variant of [`Linker::define_module_instance_unchecked`].
    pub fn define_module_instance<T: Config>(
        &mut self,
        store: &Store<T>,
        module_name: String,
        module: Stored<ModuleAddr>,
    ) -> Result<(), RuntimeError> {
        // 1. get or insert the `StoreId`
        let module_store_id = module.id().expect("this type to always contain a StoreId");
        let linker_store_id = *self.store_id.get_or_insert(module_store_id);
        if linker_store_id != module_store_id {
            return Err(RuntimeError::StoreIdMismatch);
        }
        // 2. try unwrap
        let module = module.try_unwrap_into_bare(linker_store_id)?;
        // 3. call
        self.define_module_instance_unchecked(store, module_name, module)?;
        // 4. rewrap
        // result is the unit type.
        // 5. return
        Ok(())
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
    /// Therefore, it is advised against to mix the unchecked and checked API
    /// for a single [`Linker`] instance.
    ///
    /// # Errors
    ///
    /// - [`RuntimeError::LinkerNotYetAssociatedWithStoreId`]
    /// - [`RuntimeError::UnableToResolveExternLookup`]
    pub fn get(&self, module_name: String, name: String) -> Result<StoredExternVal, RuntimeError> {
        // 1. get or insert the `StoreId`
        // TODO docs are not consistent
        let Some(linker_store_id) = self.store_id else {
            // At this point we have no way to set the current store id, because
            // the parameters are all non-stored types.

            // We also know that nothing was defined in this linker context through
            // the checked methods yet, because `self.store_id` has not been set
            // yet. Therefore, a get would always return `None`.

            // However, when an unchecked `define` method was used before, we
            // also have to return `None` here, because even if the lookup for
            // `module_name` and `name` returns something, we cannot attach a
            // store id to it.

            return Err(RuntimeError::LinkerNotYetAssociatedWithStoreId);
        };
        // 2. try unwrap
        // no stored parameters
        // 3. call
        let extern_val = self
            .get_unchecked(module_name, name)
            .ok_or(RuntimeError::UnableToResolveExternLookup)?;
        // 4. rewrap
        // Safety: The `ExternVal` just came from the current `Linker`. Because
        // a `Linker` can always be used with only one unique `Store`, this
        // `ExternVal` must be from the current Linker's store.
        let stored_extern_val = unsafe { StoredExternVal::from_bare(extern_val, linker_store_id) };
        // 5. return
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
        // 1. get or insert `StoreId`
        let Some(linker_store_id) = self.store_id else {
            // We are not able to perform safe linking (see this method's and
            // `Linker::get`'s documentations).
            return Err(RuntimeError::LinkerNotYetAssociatedWithStoreId);
        };
        // 2. try unwrap
        // no stored parameters
        // 3. call
        let extern_vals = self.instantiate_pre_unchecked(validation_info)?;
        // 4. rewrap
        // Safety: All `ExternVal`s just came from the current `Linker`. Because
        // a Linker can always be used with only one unique `Store`, all
        // `ExternVal`s must be from the current Linker's store.
        let stored_extern_vals = unsafe { wrap_vec_elements(extern_vals, linker_store_id) };
        // 5. retur
        Ok(stored_extern_vals)
    }

    /// This is a safe variant of [`Linker::module_instantiate_unchecked`].
    pub fn module_instantiate<'b, T: Config>(
        &mut self,
        store: &mut Store<'b, T>,
        validation_info: &ValidationInfo<'b>,
        maybe_fuel: Option<u32>,
    ) -> Result<StoredInstantiationOutcome, RuntimeError> {
        // 1. get or insert `StoreId`
        let linker_store_id = *self.store_id.get_or_insert(store.id);
        if linker_store_id != store.id {
            return Err(RuntimeError::StoreIdMismatch);
        }
        // 2. try unwrap
        // no stored parameters
        // 3. call
        let instantiation_outcome =
            self.module_instantiate_unchecked(store, validation_info, maybe_fuel)?;
        // 4. rewrap
        // Safety: The `InstantiationOutcome` just came from the current
        // `Linker`. Because a linker can always be used with only one unique
        // `Store`, the `InstantiationOutcome` must be from the current Linker's
        // store.
        let stored_instantiation_outcome = unsafe {
            StoredInstantiationOutcome::from_bare(instantiation_outcome, linker_store_id)
        };
        // 5. return
        Ok(stored_instantiation_outcome)
    }
}

/// A trait for types that might have a [`StoreId`] attached to them, so-called
/// _stored_ types.
trait AbstractStored: Sized {
    type BareTy: Sized;

    /// Creates a new stored object
    ///
    /// # Safety
    /// The caller has to guarantee that the bare value comes from a [`Store`]
    /// with the given [`StoreId`].
    unsafe fn from_bare(bare_value: Self::BareTy, id: StoreId) -> Self;

    /// Gets the id of this stored object
    ///
    /// Not all stored objects require to have an id attached to them.
    fn id(&self) -> Option<StoreId>;

    /// Converts this stored object into its bare form that does not have any [`StoreId`] attached to it.
    fn into_bare(self) -> Self::BareTy;

    /// Checks if this stored object comes from a specific store by its
    /// [`StoreId`]. If true, it converts self into its bare form, otherwise an
    /// error is returned.
    ///
    /// # Errors
    /// - [`RuntimeError::StoreIdMismatch`]
    fn try_unwrap_into_bare(
        self,
        expected_store_id: StoreId,
    ) -> Result<Self::BareTy, RuntimeError> {
        if let Some(id) = self.id() {
            if id != expected_store_id {
                return Err(RuntimeError::StoreIdMismatch);
            }
        }

        Ok(self.into_bare())
    }
}

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

impl<T> Stored<T> {
    // TODO remove this after the `ResumableRef` rework. Currently
    // `ResumableRef` can store data, however it should merely be an addr type
    // into the store in the future.
    fn as_mut(&mut self) -> Stored<&mut T> {
        Stored {
            id: self.id,
            inner: &mut self.inner,
        }
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
            ExternVal::Func(func_addr) => Self::Func(Stored::from_bare(func_addr, id)),
            ExternVal::Table(table_addr) => Self::Table(Stored::from_bare(table_addr, id)),
            ExternVal::Mem(mem_addr) => Self::Mem(Stored::from_bare(mem_addr, id)),
            ExternVal::Global(global_addr) => Self::Global(Stored::from_bare(global_addr, id)),
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

impl AbstractStored for StoredRunState {
    type BareTy = RunState;

    unsafe fn from_bare(bare_value: Self::BareTy, id: StoreId) -> Self {
        match bare_value {
            RunState::Finished {
                values,
                maybe_remaining_fuel,
            } => Self::Finished {
                values: wrap_vec_elements(values, id),
                maybe_remaining_fuel,
            },
            RunState::Resumable {
                resumable_ref,
                required_fuel,
            } => Self::Resumable {
                resumable_ref: Stored::from_bare(resumable_ref, id),
                required_fuel,
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
    pub maybe_remaining_fuel: Option<u32>,
}

impl AbstractStored for StoredInstantiationOutcome {
    type BareTy = InstantiationOutcome;

    unsafe fn from_bare(bare_value: Self::BareTy, id: StoreId) -> Self {
        Self {
            module_addr: Stored::from_bare(bare_value.module_addr, id),
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

/// Helper method for associating every element in a [`Vec`] with a [`StoreId`].
///
/// # Safety
///
/// It must be guaranteed that all given elements come from the [`Store`] with
/// the given [`StoreId`].
unsafe fn wrap_vec_elements<S: AbstractStored>(values: Vec<S::BareTy>, id: StoreId) -> Vec<S> {
    values
        .into_iter()
        .map(|value| {
            // Safety: The caller guarantees that all values in this Vec come
            // from the store with given id. Therefore, this is also true for
            // this specific `Value`.
            unsafe { S::from_bare(value, id) }
        })
        .collect()
}

/// Helper method for checking if all [`Value`](crate::execution::Value)s in a slice have the given
/// [`StoreId`] and then, if the check was true, converting them to a
/// [`Vec<Value>`].
///
/// # Errors
/// - [`RuntimeError::StoreIdMismatch`]
fn try_unwrap_values<S: AbstractStored>(
    stored_values: Vec<S>,
    expected_store_id: StoreId,
) -> Result<Vec<S::BareTy>, RuntimeError> {
    stored_values
        .into_iter()
        .map(|stored_value| stored_value.try_unwrap_into_bare(expected_store_id))
        .collect()
}
