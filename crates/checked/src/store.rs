use alloc::{string::String, vec::Vec};
use wasm::{
    FuncType, GlobalType, Hostcode, MemType, RuntimeError, TableType, ValidationInfo, Value,
    addrs::{FuncAddr, GlobalAddr, MemAddr, ModuleAddr, TableAddr},
    config::Config,
    resumable::{HostCallFinisher, HostResumable, WasmResumable},
};

use crate::{
    AbstractStored, StoreId, StoredRef, StoredResumable, StoredValue,
    stored_types::{Stored, StoredExternVal, StoredInstantiationOutcome, StoredRunState},
};

pub struct Store<'b, T: Config> {
    pub(crate) inner: wasm::Store<'b, T>,

    /// A unique identifier for this store. This is used to verify that stored
    /// objects belong to the current [`Store`](wasm::Store).
    pub(crate) id: StoreId,
}

impl<'b, T: Config> Store<'b, T> {
    /// Returns an immutable reference to the raw store.
    pub fn inner(&self) -> &wasm::Store<'b, T> {
        &self.inner
    }

    /// Deconstructs this checked store and returns its inner representation.
    pub fn into_inner(self) -> wasm::Store<'b, T> {
        self.inner
    }

    /// Returns the id of this store.
    pub fn id(&self) -> StoreId {
        self.id
    }
}

// All functions in this impl block must occur in the same order as they are
// defined in for the unchecked `Store` methods. Also all functions must follow
// the same implementation scheme to make sure they are only light wrappers:
//
// 1. try unwrap [stored parameter objects]
// 2. call [unchecked method]
// 3. rewrap [results into stored objects]
// 4. return [stored result objects]
impl<'b, T: Config> Store<'b, T> {
    pub fn new(user_data: T) -> Self {
        Self {
            inner: wasm::Store::new(user_data),
            id: StoreId::new(),
        }
    }

    /// This is a safe variant of [`Store::module_instantiate_unchecked`](crate::Store::module_instantiate_unchecked).
    pub fn module_instantiate(
        &mut self,
        validation_info: &ValidationInfo<'b>,
        extern_vals: Vec<StoredExternVal>,
        maybe_fuel: Option<u64>,
    ) -> Result<StoredInstantiationOutcome, RuntimeError> {
        // 1. try unwrap
        let extern_vals = extern_vals.try_unwrap_into_bare(self.id);
        // 2. call
        // SAFETY: It was just checked that the `ExternVal`s came from the
        // current store through their store ids.
        let instantiation_outcome = unsafe {
            self.inner
                .module_instantiate_unchecked(validation_info, extern_vals, maybe_fuel)
        }?;
        // 3. rewrap
        // SAFETY: The `InstantiationOutcome` just came from the current store.
        let stored_instantiation_outcome =
            unsafe { StoredInstantiationOutcome::from_bare(instantiation_outcome, self.id) };
        // 4. return
        Ok(stored_instantiation_outcome)
    }

    /// This is a safe variant of [`Store::$1`](crate::Store::$1).
    pub fn instance_export(
        &self,
        module_addr: Stored<ModuleAddr>,
        name: &str,
    ) -> Result<StoredExternVal, RuntimeError> {
        // 1. try unwrap
        let module_addr = module_addr.try_unwrap_into_bare(self.id);
        // 2. call
        // SAFETY: It was just checked that the `ModuleAddr` came from the
        // current store through its store id.
        let extern_val = unsafe { self.inner.instance_export_unchecked(module_addr, name) }?;
        // 3. rewrap
        // SAFETY: The `ExternVal` just came from the current store.
        let stored_extern_val = unsafe { StoredExternVal::from_bare(extern_val, self.id) };
        // 4. return
        Ok(stored_extern_val)
    }

    /// This is a safer variant of [`Store::func_alloc_unchecked`](crate::Store::func_alloc_unchecked). It is
    /// functionally equal, with the only difference being that this function
    /// returns a [`Stored<FuncAddr>`].
    ///
    /// # Safety
    ///
    /// The caller has to guarantee that if the [`Value`]s returned from the
    /// given host function are references, their addresses came either from the
    /// host function arguments or from the current [`Store`] object.
    ///
    /// See [`Store::func_alloc`](crate::Store::func_alloc_unchecked) for more information.
    #[allow(clippy::let_and_return)] // reason = "to follow the 1234 structure"
    pub fn func_alloc(&mut self, func_type: FuncType, hostcode: Hostcode) -> Stored<FuncAddr> {
        // 1. try unwrap
        // no stored parameters
        // 2. call
        let func_addr = self.inner.func_alloc_unchecked(func_type, hostcode);
        // 3. rewrap
        // 4. return
        // SAFETY: The function address just came from the current store.
        unsafe { Stored::from_bare(func_addr, self.id) }
    }

    /// This is a safe variant of [`Store::func_type_unchecked`](crate::Store::func_type_unchecked).
    pub fn func_type(&self, func_addr: Stored<FuncAddr>) -> FuncType {
        // 1. try unwrap
        let func_addr = func_addr.try_unwrap_into_bare(self.id);
        // 2. call
        // 3. rewrap
        // `FuncType` does not have a stored variant.
        // 4. return
        // SAFETY: It was just checked that the `FuncAddr` came from the current
        // store through its store id.
        unsafe { self.inner.func_type_unchecked(func_addr) }
    }

    /// This is a safe variant of [`Store::invoke_unchecked`](wasm::Store::invoke_unchecked).
    pub fn invoke(
        &mut self,
        func_addr: Stored<FuncAddr>,
        params: Vec<StoredValue>,
        maybe_fuel: Option<u64>,
    ) -> Result<StoredRunState, RuntimeError> {
        // 1. try unwrap
        let func_addr = func_addr.try_unwrap_into_bare(self.id);
        let params = params.try_unwrap_into_bare(self.id);
        // 2. call
        // SAFETY: It was just checked that the `FuncAddr` and any addresses in
        // the parameters came from the current store through their store ids.
        let run_state = unsafe { self.inner.invoke_unchecked(func_addr, params, maybe_fuel) }?;
        // 3. rewrap
        // SAFETY: The `RunState` just came from the current store.
        let stored_run_state = unsafe { StoredRunState::from_bare(run_state, self.id) };
        // 4. return
        Ok(stored_run_state)
    }

    /// This is a safe variant of [`Store::table_alloc_unchecked`](crate::Store::table_alloc_unchecked).
    pub fn table_alloc(
        &mut self,
        table_type: TableType,
        r#ref: StoredRef,
    ) -> Result<Stored<TableAddr>, RuntimeError> {
        // 1. try unwrap
        let r#ref = r#ref.try_unwrap_into_bare(self.id);
        // 2. call
        // SAFETY: It was just checked that any address in the reference came
        // from the current store through its store id.
        let table_addr = unsafe { self.inner.table_alloc_unchecked(table_type, r#ref) }?;
        // 3. rewrap
        // SAFETY: The `TableAddr` just came from the current store.
        let stored_table_addr = unsafe { Stored::from_bare(table_addr, self.id) };
        // 4. return
        Ok(stored_table_addr)
    }

    /// This is a safe variant of [`Store::table_type_unchecked`](crate::Store::table_type_unchecked).
    pub fn table_type(&self, table_addr: Stored<TableAddr>) -> TableType {
        // 1. try unwrap
        let table_addr = table_addr.try_unwrap_into_bare(self.id);
        // 2. call
        // 3. rewrap
        // `TableType` has no stored variant.
        // 4. return
        // SAFETY: It was just checked that the `TableAddr` came from the
        // current store through its store id.
        unsafe { self.inner.table_type_unchecked(table_addr) }
    }

    /// This is a safe variant of [`Store::table_read_unchecked`](crate::Store::table_read_unchecked).
    pub fn table_read(
        &self,
        table_addr: Stored<TableAddr>,
        i: u32,
    ) -> Result<StoredRef, RuntimeError> {
        // 1. try unwrap
        let table_addr = table_addr.try_unwrap_into_bare(self.id);
        // 2. call
        // SAFETY: It was just checked that the `TableAddr` came from the
        // current store through its store id.
        let r#ref = unsafe { self.inner.table_read_unchecked(table_addr, i) }?;
        // 3. rewrap
        // SAFETY: The `Ref` ust came from the current store.
        let stored_ref = unsafe { StoredRef::from_bare(r#ref, self.id) };
        // 4. return
        Ok(stored_ref)
    }

    /// This is a safe variant of [`Store::table_write_unchecked`](crate::Store::table_write_unchecked).
    pub fn table_write(
        &mut self,
        table_addr: Stored<TableAddr>,
        i: u32,
        r#ref: StoredRef,
    ) -> Result<(), RuntimeError> {
        // 1. try unwrap
        let table_addr = table_addr.try_unwrap_into_bare(self.id);
        let r#ref = r#ref.try_unwrap_into_bare(self.id);
        // 2. call
        // SAFETY: It was just checked that the `TableAddr` and any address in
        // the reference came from the current store through their store ids.
        unsafe { self.inner.table_write_unchecked(table_addr, i, r#ref) }?;
        // 3. rewrap
        // result is the unit type.
        // 4. return
        Ok(())
    }

    /// This is a safe variant of [`Store::table_size_unchecked`](crate::Store::table_size_unchecked).
    pub fn table_size(&self, table_addr: Stored<TableAddr>) -> u32 {
        // 1. try unwrap
        let table_addr = table_addr.try_unwrap_into_bare(self.id);
        // 2. call
        // 3. rewrap
        // table size has no stored variant.
        // 4. return
        // SAFETY: It was just checked that the `TableAddr` came from the
        // current store through its store id.
        unsafe { self.inner.table_size_unchecked(table_addr) }
    }

    /// This is a variant of [`Store::mem_alloc`](crate::Store::mem_alloc) that
    /// returns a stored object.
    #[allow(clippy::let_and_return)] // reason = "to follow the 1234 structure"
    pub fn mem_alloc(&mut self, mem_type: MemType) -> Stored<MemAddr> {
        // 1. try unwrap
        // no stored parameters
        // 2. call
        let mem_addr = self.inner.mem_alloc(mem_type);
        // 3. rewrap
        // 4. return
        // SAFETY: The `MemAddr` just came from the current store.
        unsafe { Stored::from_bare(mem_addr, self.id) }
    }

    /// This is a safe variant of [`Store::mem_type_unchecked`](crate::Store::mem_type_unchecked).
    pub fn mem_type(&self, mem_addr: Stored<MemAddr>) -> MemType {
        // 1. try unwrap
        let mem_addr = mem_addr.try_unwrap_into_bare(self.id);
        // 2. call
        // 3. rewrap
        // `MemType` does not have a stored variant.
        // 4. return
        // SAFETY: It was just checked that the `MemAddr` came from the current
        // store through its store id.
        unsafe { self.inner.mem_type_unchecked(mem_addr) }
    }

    /// This is a safe variant of [`Store::mem_read_unchecked`](crate::Store::mem_read_unchecked).
    pub fn mem_read(&self, mem_addr: Stored<MemAddr>, i: u32) -> Result<u8, RuntimeError> {
        // 1. try unwrap
        let mem_addr = mem_addr.try_unwrap_into_bare(self.id);
        // 2. call
        // SAFETY: It was just checked that the `MemAddr` came from the current
        // store through its store id.
        let byte = unsafe { self.inner.mem_read_unchecked(mem_addr, i) }?;
        // 3. rewrap
        // a single byte does not have a stored variant.
        // 4. return
        Ok(byte)
    }

    /// This is a safe variant of [`Store::mem_write_unchecked`](crate::Store::mem_write_unchecked).
    pub fn mem_write(
        &mut self,
        mem_addr: Stored<MemAddr>,
        i: u32,
        byte: u8,
    ) -> Result<(), RuntimeError> {
        // 1. try unwrap
        let mem_addr = mem_addr.try_unwrap_into_bare(self.id);
        // 2. call
        // SAFETY: It was just checked that the `MemAddr` came from the current
        // store through its store id.
        unsafe { self.inner.mem_write_unchecked(mem_addr, i, byte) }?;
        // 3. rewrap
        // result is the unit type.
        // 4. return
        Ok(())
    }

    /// This is a safe variant of [`Store::mem_size_unchecked`](crate::Store::mem_size_unchecked).
    pub fn mem_size(&self, mem_addr: Stored<MemAddr>) -> u32 {
        // 1. try unwrap
        let mem_addr = mem_addr.try_unwrap_into_bare(self.id);
        // 2. call
        // 3. rewrap
        // mem size does not have a stored variant.
        // 4. return
        // SAFETY: It was just checked that the `MemAddr` came from the current
        // store through its store id.
        unsafe { self.inner.mem_size_unchecked(mem_addr) }
    }

    /// This is a safe variant of [`Store::mem_grow_unchecked`](crate::Store::mem_grow_unchecked).
    pub fn mem_grow(&mut self, mem_addr: Stored<MemAddr>, n: u32) -> Result<(), RuntimeError> {
        // 1. try unwrap
        let mem_addr = mem_addr.try_unwrap_into_bare(self.id);
        // 2. call
        // SAFETY: It was just checked that the `MemAddr` came from the current
        // store through its store id.
        unsafe { self.inner.mem_grow_unchecked(mem_addr, n) }?;
        // 3. rewrap
        // result is the unit type.
        // 4. return
        Ok(())
    }

    /// This is a safe variant of [`Store::global_alloc_unchecked`](crate::Store::global_alloc_unchecked).
    pub fn global_alloc(
        &mut self,
        global_type: GlobalType,
        val: StoredValue,
    ) -> Result<Stored<GlobalAddr>, RuntimeError> {
        // 1. try unwrap
        let val = val.try_unwrap_into_bare(self.id);
        // 2. call
        // SAFETY: It was just checked that any address the value came from the
        // current store through its store id.
        let global_addr = unsafe { self.inner.global_alloc_unchecked(global_type, val) }?;
        // 3. rewrap
        // SAFETY: The `GlobalAddr` just came from the current store.
        let stored_global_addr = unsafe { Stored::from_bare(global_addr, self.id) };
        // 4. return
        Ok(stored_global_addr)
    }

    /// This is a safe variant of [`Store::global_type_unchecked`](crate::Store::global_type_unchecked).
    pub fn global_type(&self, global_addr: Stored<GlobalAddr>) -> Result<GlobalType, RuntimeError> {
        // 1. try unwrap
        let global_addr = global_addr.try_unwrap_into_bare(self.id);
        // 2. call
        // SAFETY: It was just checked that the `GlobalAddr` came from the
        // current store through its store id.
        let global_type = unsafe { self.inner.global_type_unchecked(global_addr) };
        // 3. rewrap
        // `GlobalType` does not have a stored variant.
        // 4. return
        Ok(global_type)
    }

    /// This is a safe variant of [`Store::global_read_unchecked`](crate::Store::global_read_unchecked).
    pub fn global_read(&self, global_addr: Stored<GlobalAddr>) -> StoredValue {
        // 1. try unwrap
        let global_addr = global_addr.try_unwrap_into_bare(self.id);
        // 2. call
        // SAFETY: It was just checked that the `GlobalAddr` came from the
        // current store through its store id.
        let value = unsafe { self.inner.global_read_unchecked(global_addr) };
        // 3. rewrap
        // 4. return
        // SAFETY: The `Value` just came from the current store.
        unsafe { StoredValue::from_bare(value, self.id) }
    }

    /// This is a safe variant of [`Store::global_write_unchecked`](crate::Store::global_write_unchecked).
    pub fn global_write(
        &mut self,
        global_addr: Stored<GlobalAddr>,
        val: StoredValue,
    ) -> Result<(), RuntimeError> {
        // 1. try unwrap
        let global_addr = global_addr.try_unwrap_into_bare(self.id);
        let val = val.try_unwrap_into_bare(self.id);
        // 2. call
        // SAFETY: It was just checked that the `GlobalAddr` any any address
        // contained in the value came from the current store through their
        // store ids.
        unsafe { self.inner.global_write_unchecked(global_addr, val) }?;
        // 3. rewrap
        // result is the unit type.
        // 4. return
        Ok(())
    }

    /// This is a safe variant of [`Store::create_resumable_unchecked`](crate::Store::create_resumable_unchecked).
    pub fn create_resumable(
        &self,
        func_addr: Stored<FuncAddr>,
        params: Vec<StoredValue>,
        maybe_fuel: Option<u64>,
    ) -> Result<StoredResumable<T>, RuntimeError> {
        // 1. try unwrap
        let func_addr = func_addr.try_unwrap_into_bare(self.id);
        let params = params.try_unwrap_into_bare(self.id);
        // 2. call
        // SAFETY: It was just checked that the `FuncAddr` any any addresses
        // contained in the parameters came from the current store through their
        // store ids.
        let resumable = unsafe {
            self.inner
                .create_resumable_unchecked(func_addr, params, maybe_fuel)
        }?;
        // 3. rewrap
        // SAFETY: The `Resumable` just came from the current store.
        let stored_resumable = unsafe { StoredResumable::from_bare(resumable, self.id) };
        // 4. return
        Ok(stored_resumable)
    }

    /// This is a safe variant of [`Store::resume_unchecked`](wasm::Store::resume_unchecked).
    pub fn resume(
        &mut self,
        resumable: Stored<WasmResumable>,
    ) -> Result<StoredRunState, RuntimeError> {
        // 1. try unwrap
        let resumable = resumable.try_unwrap_into_bare(self.id);
        // 2. call
        // SAFETY: It was just checked that the `WasmResumable` came from the
        // current store through its store id.
        let run_state = unsafe { self.inner.resume_unchecked(resumable) }?;
        // 3. rewrap
        // SAFETY: The `RunState` just came from the current store.
        let stored_run_state = unsafe { StoredRunState::from_bare(run_state, self.id) };
        // 4. return
        Ok(stored_run_state)
    }

    /// This is a safe variant of
    /// [`Store::finish_host_call_unchecked`](wasm::Store::finish_host_call_unchecked).
    pub fn finish_host_call(
        &mut self,
        host_call_finisher: Stored<HostCallFinisher>,
        host_call_return_values: Vec<StoredValue>,
    ) -> Result<Vec<StoredValue>, RuntimeError> {
        // 1. try unwrap
        let host_call_finisher = host_call_finisher.try_unwrap_into_bare(self.id);
        let host_call_return_values = host_call_return_values.try_unwrap_into_bare(self.id);
        // 2. call
        // SAFETY: It was just checked that the `HostCallFinisher` and all
        // `Value`s came from the current store through their store ids.
        let return_values = unsafe {
            self.inner
                .finish_host_call_unchecked(host_call_finisher, host_call_return_values)
        }?;
        // 3. rewrap
        // SAFETY: The `Value`s just came from the current store.
        let stored_return_values = unsafe { Vec::from_bare(return_values, self.id) };
        // 4. return
        Ok(stored_return_values)
    }

    /// This is a safe variant of [`Store::finish_host_call_into_resumable_unchecked`](wasm::Store::finish_host_call_into_resumable_unchecked)
    pub unsafe fn finish_host_call_into_resumable(
        &mut self,
        host_resumable: Stored<HostResumable>,
        host_call_return_values: Vec<StoredValue>,
    ) -> Result<Stored<WasmResumable>, RuntimeError> {
        // 1. try unwrap
        let host_resumable = host_resumable.try_unwrap_into_bare(self.id);
        let host_call_return_values = host_call_return_values.try_unwrap_into_bare(self.id);
        // 2. call
        // SAFETY: It was just checked that the `HostResumable` and all `Value`s
        // came from the current store through their store ids.
        let resumable = unsafe {
            self.inner
                .finish_host_call_into_resumable_unchecked(host_resumable, host_call_return_values)
        }?;
        // 3. rewrap
        // SAFETY: The `WasmResumable` just came from the current store.
        let stored_resumable = unsafe { Stored::from_bare(resumable, self.id) };
        // 4. return
        Ok(stored_resumable)
    }

    // /// This is a safe variant of [`Store::invoke_without_fuel_unchecked`](crate::Store::invoke_without_fuel_unchecked).
    // pub fn invoke_without_fuel(
    //     &mut self,
    //     func_addr: Stored<FuncAddr>,
    //     params: Vec<StoredValue>,
    // ) -> Result<Vec<StoredValue>, RuntimeError> {
    //     // 1. try unwrap
    //     let func_addr = func_addr.try_unwrap_into_bare(self.id);
    //     let params = params.try_unwrap_into_bare(self.id);
    //     // 2. call
    //     // SAFETY: It was just checked that the `FuncAddr` and any addresses
    //     // contained in the parameters came from the current store through their
    //     // store ids.
    //     let returns = unsafe { self.inner.invoke_without_fuel_unchecked(func_addr, params) }?;
    //     // 3. rewrap
    //     // SAFETY: All `Value`s just came from the current store.
    //     let returns = unsafe { Vec::from_bare(returns, self.id) };
    //     // 4. return
    //     Ok(returns)
    // }

    /// This is a safe variant of [`Store::mem_access_mut_slice_unchecked`](crate::Store::mem_access_mut_slice_unchecked).
    pub fn mem_access_mut_slice<R>(
        &self,
        memory: Stored<MemAddr>,
        accessor: impl FnOnce(&mut [u8]) -> R,
    ) -> R {
        // 1. try unwrap
        let memory = memory.try_unwrap_into_bare(self.id);
        // 2. call
        // 3. rewrap
        // result is generic
        // 4. return
        // SAFETY: It was just checked that the `MemAddr` came from the current
        // store through its store id.
        unsafe { self.inner.mem_access_mut_slice_unchecked(memory, accessor) }
    }

    /// This is a safe variant of [`Store::instance_exports_unchecked`](crate::Store::instance_exports_unchecked)
    pub fn instance_exports(
        &self,
        module_addr: Stored<ModuleAddr>,
    ) -> Vec<(String, StoredExternVal)> {
        // 1. try unwrap
        let module_addr = module_addr.try_unwrap_into_bare(self.id);
        // 2. call
        // SAFETY: We just checked that this module address is valid in the
        // current store through its store id.
        let exports = unsafe { self.inner.instance_exports_unchecked(module_addr) };
        // 3. rewrap
        // 4. return
        exports
            .into_iter()
            .map(|(name, externval)| {
                // SAFETY: The `ExternVal`s just came from the current store.
                let stored_externval = unsafe { StoredExternVal::from_bare(externval, self.id) };
                (name, stored_externval)
            })
            .collect()
    }
}
