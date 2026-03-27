//! A simple registry for managing host hunctions/calls.
//!
//! Contrary to the other crates, this crate does not provide a re-useable
//! general-purpose implementation. Instead, this implementation is used for
//! tests and the specificaiton testsuite runner.

#![no_std]
#![deny(
    clippy::missing_safety_doc,
    clippy::undocumented_unsafe_blocks,
    unsafe_op_in_unsafe_fn
)]

extern crate alloc;

use alloc::{borrow::ToOwned, boxed::Box, vec::Vec};

use checked::{Store, Stored, StoredHostCall, StoredInteropValueList, StoredRunState, StoredValue};
use wasm::{
    addrs::FuncAddr, config::Config, resumable::HostResumable, value::ValueTypeMismatchError,
    FuncType, ResultType, RuntimeError,
};

type BoxedHostFn<T> = Box<dyn FnMut(&mut T, Vec<StoredValue>) -> Vec<StoredValue>>;

/// A simple registry for host functions.
///
/// # Features
///
/// - based on the [`checked API`](checked) and its `interop` feature
/// - host functions may access generic user data `T`
/// - boxed and dynamically dispatched host functions
pub struct Registry<T> {
    host_functions: Vec<BoxedHostFn<T>>,
    next_hostcode: usize,
}

impl<T> Default for Registry<T> {
    fn default() -> Self {
        Self {
            host_functions: Vec::new(),
            next_hostcode: 0,
        }
    }
}

impl<T> Registry<T> {
    pub fn alloc_host_function<C: Config>(
        &mut self,
        store: &mut Store<C>,
        func_type: FuncType,
        host_function: impl FnMut(&mut T, Vec<StoredValue>) -> Vec<StoredValue> + 'static,
    ) -> Stored<FuncAddr> {
        let hostcode = self.next_hostcode;
        self.next_hostcode += 1;

        self.host_functions.push(Box::new(host_function));

        store.func_alloc(func_type, hostcode)
    }

    pub fn alloc_host_function_typed<C, Params, Returns, F>(
        &mut self,
        store: &mut Store<C>,
        mut host_function: F,
    ) -> Stored<FuncAddr>
    where
        C: Config,
        Params: StoredInteropValueList,
        Returns: StoredInteropValueList,
        F: FnMut(&mut T, Params) -> Returns + 'static,
    {
        let func_type = FuncType {
            params: ResultType {
                valtypes: Params::TYS.to_owned(),
            },
            returns: ResultType {
                valtypes: Returns::TYS.to_owned(),
            },
        };

        self.alloc_host_function(store, func_type, move |user_data, params| {
            let params = Params::try_from_values(params.into_iter()).unwrap();
            let returns = host_function(user_data, params);
            returns.into_values()
        })
    }

    pub fn invoke_without_fuel<C: Config>(
        &mut self,
        user_data: &mut T,
        store: &mut Store<C>,
        func_addr: Stored<FuncAddr>,
        params: Vec<StoredValue>,
    ) -> Result<Vec<StoredValue>, RuntimeError> {
        let resumable = store.create_resumable(func_addr, params, None)?;
        let mut run_state = store.resume(resumable)?;
        loop {
            match run_state {
                StoredRunState::Finished { values, .. } => return Ok(values),
                StoredRunState::Resumable {
                    resumable,
                    required_fuel,
                } => {
                    assert!(required_fuel.is_none(), "fuel is disabled");
                    run_state = store.resume_wasm(resumable)?;
                }
                StoredRunState::HostCalled {
                    host_call,
                    resumable,
                } => {
                    run_state = self.perform_host_call(user_data, store, host_call, resumable)?;
                }
            }
        }
    }

    pub fn invoke_without_fuel_typed<C, Params, Returns>(
        &mut self,
        user_data: &mut T,
        store: &mut Store<C>,
        func_addr: Stored<FuncAddr>,
        params: Params,
    ) -> Result<Returns, RuntimeError>
    where
        C: Config,
        Params: StoredInteropValueList,
        Returns: StoredInteropValueList,
    {
        let params = params.into_values();
        let returns = self.invoke_without_fuel(user_data, store, func_addr, params)?;
        Returns::try_from_values(returns.into_iter())
            .map_err(|ValueTypeMismatchError| RuntimeError::FunctionInvocationSignatureMismatch)
    }

    pub fn perform_host_call<C: Config>(
        &mut self,
        user_data: &mut T,
        store: &mut Store<C>,
        host_call: StoredHostCall,
        host_resumable: Stored<HostResumable>,
    ) -> Result<StoredRunState, RuntimeError> {
        let host_function = &mut self.host_functions[host_call.hostcode];
        let returns = host_function(user_data, host_call.params);
        store.finish_host_call(host_resumable, returns)
    }
}
