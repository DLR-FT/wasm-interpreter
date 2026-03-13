#![no_std]

use alloc::{borrow::ToOwned, boxed::Box, collections::btree_map::BTreeMap, vec::Vec};
use interop::InteropValueList;
use wasm::{
    FuncType, ResultType, Store, Value,
    addrs::FuncAddr,
    config::Config,
    resumable::{HostCall, HostResumable, RunState},
};

extern crate alloc;

/// A registry for host functions.
pub struct Registry<HostFn> {
    host_functions: BTreeMap<usize, HostFn>,
    next_hostcode: usize,
}

impl<HostFn> Default for Registry<HostFn> {
    fn default() -> Self {
        Self {
            host_functions: BTreeMap::default(),
            next_hostcode: 0,
        }
    }
}

impl<HostFn> Registry<HostFn>
where
    HostFn: FnMut(Vec<Value>) -> Vec<Value>,
{
    pub fn alloc_host_function<T: Config>(
        &mut self,
        store: &mut Store<T>,
        func_type: FuncType,
        host_function: HostFn,
    ) -> FuncAddr {
        let hostcode = self.next_hostcode;
        self.next_hostcode += 1;
        self.host_functions.insert(hostcode, host_function);
        store.func_alloc_unchecked(func_type, hostcode)
    }

    pub fn perform_host_call<T: Config>(
        &mut self,
        store: &mut Store<T>,
        host_call: HostCall,
        host_resumable: HostResumable,
    ) -> RunState {
        let host_function = self.host_functions.get_mut(&host_call.hostcode).unwrap();

        let returns = host_function(host_call.params);

        unsafe { store.finish_host_call(host_resumable, returns).unwrap() }
    }
}

impl Registry<Box<dyn FnMut(Vec<Value>) -> Vec<Value>>> {
    pub fn alloc_host_function_typed<T, Params, Returns, F>(
        &mut self,
        store: &mut Store<T>,
        host_function: F,
    ) -> FuncAddr
    where
        T: Config,
        Params: InteropValueList,
        Returns: InteropValueList,
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

        self.alloc_host_function(
            store,
            func_type,
            Box::new(move |params| {
                let params = Params::try_from_values(params.into_iter()).unwrap();
                let returns = host_function(params);
                returns.into_values()
            }),
        )
    }
}
