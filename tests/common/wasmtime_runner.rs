pub struct WASMTimeRunner<T> {
    instance: wasmtime::Instance,
    store: wasmtime::Store<T>
}

impl<T> WASMTimeRunner<T> {
    pub fn new(wat: &str, initial_store: T) -> wasmtime::Result<Self> {
        let engine = wasmtime::Engine::default();
        let module = wasmtime::Module::new(&engine, wat)?;
        let linker = wasmtime::Linker::new(&engine);

        let mut store = wasmtime::Store::new(&engine, initial_store);
        let instance = linker.instantiate(&mut store, &module)?;
        
        Ok(WASMTimeRunner {
            instance,
            store
        })
    }

    pub fn execute<WTParams, WTReturns>(
        &mut self,
        func_name: &str,
        params: WTParams,
    ) -> wasmtime::Result<WTReturns>
    where
        WTParams: wasmtime::WasmParams,
        WTReturns: wasmtime::WasmResults,
    {
        // use wasmtime::*;

        let function = self.instance.get_typed_func::<WTParams, WTReturns>(&mut self.store, func_name)?;

        function.call(&mut self.store, params)
    }
}
