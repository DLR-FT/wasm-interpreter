use std::collections::HashMap;

use wasm::{
    addrs::FuncAddr,
    checked::{Stored, StoredHostCall, StoredInteropValueList, StoredValue},
    config::Config,
    resumable::{HostResumable, WasmResumable},
    FuncType, ResultType, Store,
};

/// A registry for boxed dynamically-dispatched host functions.
#[derive(Default)]
pub struct Registry {
    host_functions: HashMap<usize, Box<dyn Fn(Vec<StoredValue>) -> Vec<StoredValue>>>,
    next_hostcode: usize,
}

impl Registry {
    pub fn alloc_host_function<T: Config>(
        &mut self,
        store: &mut Store<T>,
        func_type: FuncType,
        host_function: impl Fn(Vec<StoredValue>) -> Vec<StoredValue> + 'static,
    ) -> Stored<FuncAddr> {
        let hostcode = self.next_hostcode;
        self.next_hostcode += 1;

        self.host_functions
            .insert(hostcode, Box::new(host_function));

        store.func_alloc(func_type, hostcode)
    }

    pub fn alloc_host_function_typed<T, Params, Returns, F>(
        &mut self,
        store: &mut Store<T>,
        host_function: F,
    ) -> Stored<FuncAddr>
    where
        T: Config,
        Params: StoredInteropValueList,
        Returns: StoredInteropValueList,
        F: Fn(Params) -> Returns + 'static,
    {
        let func_type = FuncType {
            params: ResultType {
                valtypes: Params::TYS.to_owned(),
            },
            returns: ResultType {
                valtypes: Returns::TYS.to_owned(),
            },
        };

        self.alloc_host_function(store, func_type, move |params| {
            let params = Params::try_from_values(params.into_iter()).unwrap();
            let returns = host_function(params);
            returns.into_values()
        })
    }

    pub fn perform_host_call_into_resumable<T: Config>(
        &self,
        store: &mut Store<T>,
        host_call: StoredHostCall,
        host_resumable: Stored<HostResumable>,
    ) -> Stored<WasmResumable> {
        let host_function = self.host_functions.get(&host_call.hostcode).unwrap();

        let returns = host_function(host_call.params);

        store
            .finish_host_call_into_resumable(host_resumable, returns)
            .unwrap()
    }
}
