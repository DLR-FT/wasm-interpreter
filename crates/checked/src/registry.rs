use alloc::{boxed::Box, vec::Vec};
use wasm::{FuncType, addrs::FuncAddr, config::Config, resumable::HostResumable};

use crate::{Store, StoreId, Stored, StoredHostCall, StoredRunState, StoredValue};

struct Registry {
    inner: registry::Registry<Box<dyn FnMut(Vec<StoredValue>) -> Vec<StoredValue>>>,
    store_id: StoreId,
}

impl Registry {
    pub fn alloc_host_function<T: Config>(
        &mut self,
        store: &mut Store<T>,
        func_type: FuncType,
        host_function: impl FnMut(Vec<StoredValue>),
    ) -> Stored<FuncAddr> {
        self.inner.alloc_host_function();

        // let hostcode = self.next_hostcode;
        // self.next_hostcode += 1;
        // self.host_functions.insert(hostcode, host_function);
        // store.func_alloc_unchecked(func_type, hostcode)
    }

    pub fn perform_host_call<T: Config>(
        &mut self,
        store: &mut Store<T>,
        host_call: StoredHostCall,
        host_resumable: Stored<HostResumable>,
    ) -> StoredRunState {
        let host_function = self.host_functions.get_mut(&host_call.hostcode).unwrap();

        let returns = host_function(host_call.params);

        unsafe { store.finish_host_call(host_resumable, returns).unwrap() }
    }
}

#[cfg(feature = "interop")]
pub mod interop {
    use alloc::{borrow::ToOwned, boxed::Box, vec::Vec};
    use wasm::{FuncType, ResultType, addrs::FuncAddr, config::Config};

    use crate::{Store, Stored, StoredInteropValueList, StoredValue, registry::Registry};

    impl Registry<Box<dyn FnMut(Vec<StoredValue>) -> Vec<StoredValue>>> {
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
}
