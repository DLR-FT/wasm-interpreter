impl<'b, T: Config> Store<'b, T> {
    // Note: `pub fn new_checked()` is missing, because it does not interact
    // with any stored objects.

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

    // Note: `pub fn func_alloc_checked(&mut self, ...)` is missing, because it would
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

    // Note: `pub fn func_alloc_typed(&mut self, ...)` is missing, because of
    // the same reason `func_alloc` is missing.

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

    pub fn invoke_typed_without_fuel_checked<
        Params: StoredInteropValueList,
        Returns: StoredInteropValueList,
    >(
        &mut self,
        function: Stored<FuncAddr>,
        params: Params,
    ) -> Result<Returns, RuntimeError> {
        self.invoke_without_fuel_checked(function, params.into_values())
            .and_then(|results| {
                Returns::try_from_values(results.into_iter()).map_err(|ValueTypeMismatchError| {
                    RuntimeError::FunctionInvocationSignatureMismatch
                })
            })
    }
}
