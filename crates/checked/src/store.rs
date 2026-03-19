use alloc::{string::String, vec::Vec};
use wasm::{
    addrs::{FuncAddr, GlobalAddr, MemAddr, ModuleAddr, TableAddr},
    config::Config,
    resumable::{HostResumable, WasmResumable},
    FuncType, GlobalType, HaltExecutionError, MemType, RuntimeError, TableType, ValidationInfo,
    Value,
};

use crate::{
    stored_types::{Stored, StoredExternVal, StoredInstantiationOutcome, StoredRunState},
    AbstractStored, StoreId, StoredRef, StoredResumable, StoredValue,
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

    /// This is a safe variant of
    /// [`Store::module_instantiate`](wasm::Store::module_instantiate).
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
                .module_instantiate(validation_info, extern_vals, maybe_fuel)
        }?;
        // 3. rewrap
        // SAFETY: The `InstantiationOutcome` just came from the current store.
        let stored_instantiation_outcome =
            unsafe { StoredInstantiationOutcome::from_bare(instantiation_outcome, self.id) };
        // 4. return
        Ok(stored_instantiation_outcome)
    }

    /// This is a safe variant of
    /// [`Store::instance_export`](wasm::Store::instance_export).
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
        let extern_val = unsafe { self.inner.instance_export(module_addr, name) }?;
        // 3. rewrap
        // SAFETY: The `ExternVal` just came from the current store.
        let stored_extern_val = unsafe { StoredExternVal::from_bare(extern_val, self.id) };
        // 4. return
        Ok(stored_extern_val)
    }

    /// This is a safer variant of
    /// [`Store::func_alloc`](wasm::Store::func_alloc). It is functionally
    /// equal, with the only difference being that this function returns a
    /// [`Stored<FuncAddr>`].
    ///
    /// # Safety
    ///
    /// The caller has to guarantee that if the [`Value`]s returned from the
    /// given host function are references, their addresses came either from the
    /// host function arguments or from the current [`Store`] object.
    ///
    /// See [`Store::func_alloc`](wasm::Store::func_alloc) for more information.
    #[allow(clippy::let_and_return)] // reason = "to follow the 1234 structure"
    pub unsafe fn func_alloc(
        &mut self,
        func_type: FuncType,
        host_func: fn(&mut T, Vec<Value>) -> Result<Vec<Value>, HaltExecutionError>,
    ) -> Stored<FuncAddr> {
        // 1. try unwrap
        // no stored parameters
        // 2. call
        // SAFETY: The caller ensures that if the host function returns
        // references, they originate either from the arguments or the current
        // store.
        let func_addr = unsafe { self.inner.func_alloc(func_type, host_func) };
        // 3. rewrap
        // 4. return
        // SAFETY: The function address just came from the current store.
        unsafe { Stored::from_bare(func_addr, self.id) }
    }

    /// This is a safe variant of [`Store::func_type`](wasm::Store::func_type).
    pub fn func_type(&self, func_addr: Stored<FuncAddr>) -> FuncType {
        // 1. try unwrap
        let func_addr = func_addr.try_unwrap_into_bare(self.id);
        // 2. call
        // 3. rewrap
        // `FuncType` does not have a stored variant.
        // 4. return
        // SAFETY: It was just checked that the `FuncAddr` came from the current
        // store through its store id.
        unsafe { self.inner.func_type(func_addr) }
    }

    /// This is a safe variant of [`Store::invoke`](wasm::Store::invoke).
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
        let run_state = unsafe { self.inner.invoke(func_addr, params, maybe_fuel) }?;
        // 3. rewrap
        // SAFETY: The `RunState` just came from the current store.
        let stored_run_state = unsafe { StoredRunState::from_bare(run_state, self.id) };
        // 4. return
        Ok(stored_run_state)
    }

    /// This is a safe variant of
    /// [`Store::table_alloc`](wasm::Store::table_alloc).
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
        let table_addr = unsafe { self.inner.table_alloc(table_type, r#ref) }?;
        // 3. rewrap
        // SAFETY: The `TableAddr` just came from the current store.
        let stored_table_addr = unsafe { Stored::from_bare(table_addr, self.id) };
        // 4. return
        Ok(stored_table_addr)
    }

    /// This is a safe variant of
    /// [`Store::table_type`](wasm::Store::table_type).
    pub fn table_type(&self, table_addr: Stored<TableAddr>) -> TableType {
        // 1. try unwrap
        let table_addr = table_addr.try_unwrap_into_bare(self.id);
        // 2. call
        // 3. rewrap
        // `TableType` has no stored variant.
        // 4. return
        // SAFETY: It was just checked that the `TableAddr` came from the
        // current store through its store id.
        unsafe { self.inner.table_type(table_addr) }
    }

    /// This is a safe variant of
    /// [`Store::table_read`](wasm::Store::table_read).
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
        let r#ref = unsafe { self.inner.table_read(table_addr, i) }?;
        // 3. rewrap
        // SAFETY: The `Ref` ust came from the current store.
        let stored_ref = unsafe { StoredRef::from_bare(r#ref, self.id) };
        // 4. return
        Ok(stored_ref)
    }

    /// This is a safe variant of
    /// [`Store::table_write`](wasm::Store::table_write).
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
        unsafe { self.inner.table_write(table_addr, i, r#ref) }?;
        // 3. rewrap
        // result is the unit type.
        // 4. return
        Ok(())
    }

    /// This is a safe variant of
    /// [`Store::table_size`](wasm::Store::table_size).
    pub fn table_size(&self, table_addr: Stored<TableAddr>) -> u32 {
        // 1. try unwrap
        let table_addr = table_addr.try_unwrap_into_bare(self.id);
        // 2. call
        // 3. rewrap
        // table size has no stored variant.
        // 4. return
        // SAFETY: It was just checked that the `TableAddr` came from the
        // current store through its store id.
        unsafe { self.inner.table_size(table_addr) }
    }

    /// This is a variant of [`Store::mem_alloc`](wasm::Store::mem_alloc) that
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

    /// This is a safe variant of [`Store::mem_type`](wasm::Store::mem_type).
    pub fn mem_type(&self, mem_addr: Stored<MemAddr>) -> MemType {
        // 1. try unwrap
        let mem_addr = mem_addr.try_unwrap_into_bare(self.id);
        // 2. call
        // 3. rewrap
        // `MemType` does not have a stored variant.
        // 4. return
        // SAFETY: It was just checked that the `MemAddr` came from the current
        // store through its store id.
        unsafe { self.inner.mem_type(mem_addr) }
    }

    /// This is a safe variant of [`Store::mem_read`](wasm::Store::mem_read).
    pub fn mem_read(&self, mem_addr: Stored<MemAddr>, i: u32) -> Result<u8, RuntimeError> {
        // 1. try unwrap
        let mem_addr = mem_addr.try_unwrap_into_bare(self.id);
        // 2. call
        // SAFETY: It was just checked that the `MemAddr` came from the current
        // store through its store id.
        let byte = unsafe { self.inner.mem_read(mem_addr, i) }?;
        // 3. rewrap
        // a single byte does not have a stored variant.
        // 4. return
        Ok(byte)
    }

    /// This is a safe variant of [`Store::mem_write`](wasm::Store::mem_write).
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
        unsafe { self.inner.mem_write(mem_addr, i, byte) }?;
        // 3. rewrap
        // result is the unit type.
        // 4. return
        Ok(())
    }

    /// This is a safe variant of [`Store::mem_size`](wasm::Store::mem_size).
    pub fn mem_size(&self, mem_addr: Stored<MemAddr>) -> u32 {
        // 1. try unwrap
        let mem_addr = mem_addr.try_unwrap_into_bare(self.id);
        // 2. call
        // 3. rewrap
        // mem size does not have a stored variant.
        // 4. return
        // SAFETY: It was just checked that the `MemAddr` came from the current
        // store through its store id.
        unsafe { self.inner.mem_size(mem_addr) }
    }

    /// This is a safe variant of [`Store::mem_grow`](wasm::Store::mem_grow).
    pub fn mem_grow(&mut self, mem_addr: Stored<MemAddr>, n: u32) -> Result<(), RuntimeError> {
        // 1. try unwrap
        let mem_addr = mem_addr.try_unwrap_into_bare(self.id);
        // 2. call
        // SAFETY: It was just checked that the `MemAddr` came from the current
        // store through its store id.
        unsafe { self.inner.mem_grow(mem_addr, n) }?;
        // 3. rewrap
        // result is the unit type.
        // 4. return
        Ok(())
    }

    /// This is a safe variant of
    /// [`Store::global_alloc`](wasm::Store::global_alloc).
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
        let global_addr = unsafe { self.inner.global_alloc(global_type, val) }?;
        // 3. rewrap
        // SAFETY: The `GlobalAddr` just came from the current store.
        let stored_global_addr = unsafe { Stored::from_bare(global_addr, self.id) };
        // 4. return
        Ok(stored_global_addr)
    }

    /// This is a safe variant of
    /// [`Store::global_type`](wasm::Store::global_type).
    pub fn global_type(&self, global_addr: Stored<GlobalAddr>) -> Result<GlobalType, RuntimeError> {
        // 1. try unwrap
        let global_addr = global_addr.try_unwrap_into_bare(self.id);
        // 2. call
        // SAFETY: It was just checked that the `GlobalAddr` came from the
        // current store through its store id.
        let global_type = unsafe { self.inner.global_type(global_addr) };
        // 3. rewrap
        // `GlobalType` does not have a stored variant.
        // 4. return
        Ok(global_type)
    }

    /// This is a safe variant of
    /// [`Store::global_read`](wasm::Store::global_read).
    pub fn global_read(&self, global_addr: Stored<GlobalAddr>) -> StoredValue {
        // 1. try unwrap
        let global_addr = global_addr.try_unwrap_into_bare(self.id);
        // 2. call
        // SAFETY: It was just checked that the `GlobalAddr` came from the
        // current store through its store id.
        let value = unsafe { self.inner.global_read(global_addr) };
        // 3. rewrap
        // 4. return
        // SAFETY: The `Value` just came from the current store.
        unsafe { StoredValue::from_bare(value, self.id) }
    }

    /// This is a safe variant of
    /// [`Store::global_write`](wasm::Store::global_write).
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
        unsafe { self.inner.global_write(global_addr, val) }?;
        // 3. rewrap
        // result is the unit type.
        // 4. return
        Ok(())
    }

    /// This is a safe variant of
    /// [`Store::create_resumable`](wasm::Store::create_resumable).
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
        let resumable = unsafe { self.inner.create_resumable(func_addr, params, maybe_fuel) }?;
        // 3. rewrap
        // SAFETY: The `Resumable` just came from the current store.
        let stored_resumable = unsafe { StoredResumable::from_bare(resumable, self.id) };
        // 4. return
        Ok(stored_resumable)
    }

    /// This is a safe variant of [`Store::resume`](wasm::Store::resume).
    pub fn resume(
        &mut self,
        resumable: Stored<WasmResumable>,
    ) -> Result<StoredRunState, RuntimeError> {
        // 1. try unwrap
        let resumable = resumable.try_unwrap_into_bare(self.id);
        // 2. call
        // SAFETY: It was just checked that the `WasmResumable` came from the
        // current store through its store id.
        let run_state = unsafe { self.inner.resume(resumable) }?;
        // 3. rewrap
        // SAFETY: The `RunState` just came from the current store.
        let stored_run_state = unsafe { StoredRunState::from_bare(run_state, self.id) };
        // 4. return
        Ok(stored_run_state)
    }

    /// This is a safe variant of [`wasm::Store::resume_host`].
    pub fn resume_host(
        &mut self,
        resumable: Stored<HostResumable<T>>,
    ) -> Result<Vec<StoredValue>, RuntimeError> {
        // 1. try unwrap
        let resumable = resumable.try_unwrap_into_bare(self.id);
        // 2. call
        // SAFETY: It was just checked that the `HostResumable` came from the
        // current store through its store id.
        let values = unsafe { self.inner.resume_host(resumable) }?;
        // 3. rewrap
        // SAFETY: All `Value`s just came from the current store.
        let stored_values = unsafe { Vec::from_bare(values, self.id) };
        // 4. return
        Ok(stored_values)
    }

    /// This is a safe variant of
    /// [`Store::invoke_simple`](wasm::Store::invoke_simple)
    pub fn invoke_simple(
        &mut self,
        function: Stored<FuncAddr>,
        params: Vec<StoredValue>,
    ) -> Result<Vec<StoredValue>, RuntimeError> {
        // 1. try unwrap
        let function = function.try_unwrap_into_bare(self.id);
        let params = params.try_unwrap_into_bare(self.id);
        // 2. call
        // SAFETY: It was just checked that the `FuncAddr` and all `Value`s came
        // from the current store through their store ids.
        let return_values = unsafe { self.inner.invoke_simple(function, params) }?;
        // 3. rewrap
        // SAFETY: The `Value`s just came from the current store.
        let stored_return_values = unsafe { Vec::from_bare(return_values, self.id) };
        // 4. return
        Ok(stored_return_values)
    }

    /// This is a safe variant of
    /// [`Store::mem_access_mut_slice`](wasm::Store::mem_access_mut_slice).
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
        unsafe { self.inner.mem_access_mut_slice(memory, accessor) }
    }

    /// This is a safe variant of
    /// [`Store::instance_exports`](wasm::Store::instance_exports)
    pub fn instance_exports(
        &self,
        module_addr: Stored<ModuleAddr>,
    ) -> Vec<(String, StoredExternVal)> {
        // 1. try unwrap
        let module_addr = module_addr.try_unwrap_into_bare(self.id);
        // 2. call
        // SAFETY: We just checked that this module address is valid in the
        // current store through its store id.
        let exports = unsafe { self.inner.instance_exports(module_addr) };
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
