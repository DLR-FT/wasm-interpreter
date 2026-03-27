//! TODO

use core::num::NonZeroU64;

use alloc::vec::Vec;

use crate::{addrs::FuncAddr, value_stack::Stack, Hostcode, Value};

/// A [`WasmResumable`] is an object used to resume execution of Wasm code.
///
/// # Safety
///
/// TODO:
///
/// - stack must be initialized with correct parameters to function
///   referenced by function address
/// - program counter must be valid for the bytecode of function referenced by the function address
/// - stp must point to the correct sidetable entry for the function referenced by function address
#[derive(Debug)]
pub struct WasmResumable {
    pub(crate) stack: Stack,
    pub(crate) pc: usize,
    pub(crate) stp: usize,
    pub(crate) current_func_addr: FuncAddr,
    pub(crate) maybe_fuel: Option<u64>,
}

impl WasmResumable {
    pub fn fuel(&self) -> Option<u64> {
        self.maybe_fuel
    }

    pub fn fuel_mut(&mut self) -> &mut Option<u64> {
        &mut self.maybe_fuel
    }
}

/// A [`HostCall`] object contains information required for executing a specific
/// host function.
#[derive(Clone, Debug)]
pub struct HostCall {
    /// Must contain the correct parameter types for the host function with host
    /// code `hostcode`.
    pub params: Vec<Value>,
    pub hostcode: Hostcode,
}

/// A [`HostResumable`] is used to resume execution after executing its
/// [`HostCall`].
///
/// When a host function is called, a [`HostResumable`] and [`HostCall`] are
/// returned. After the [`HostCall`] was used to execute the host function, the
/// [`HostResumable`] is used together with the return values of the host call
/// to resume execution.
#[derive(Debug)]
pub struct HostResumable {
    pub(crate) host_func_addr: FuncAddr,
    pub(crate) inner_resumable: Option<WasmResumable>,
    /// Hack: This is `Some` only if `inner_resumable` is `None`. In that case
    /// it is used to store the maybe_fuel, so it can be returned in
    /// [`RunState::Finished`] later.
    pub(crate) maybe_fuel: Option<Option<u64>>,
}

#[derive(Debug)]
pub enum Resumable {
    Wasm(WasmResumable),
    Host {
        host_call: HostCall,
        host_resumable: HostResumable,
    },
}

impl Resumable {
    /// Tries to convert this [`Resumable`] into a [`WasmResumable`]
    pub fn as_wasm(self) -> Option<WasmResumable> {
        match self {
            Self::Wasm(wasm_resumable) => Some(wasm_resumable),
            Self::Host { .. } => None,
        }
    }

    /// Tries to convert this [`Resumable`] into a [`HostCall`] and
    /// [`HostResumable`]
    pub fn as_host(self) -> Option<(HostCall, HostResumable)> {
        match self {
            Self::Wasm(_) => None,
            Self::Host {
                host_call,
                host_resumable,
            } => Some((host_call, host_resumable)),
        }
    }
}

/// Represents the state of a possibly interrupted resumable.
pub enum RunState {
    /// represents a resumable that has executed completely with return values `values` and possibly remaining fuel
    /// `maybe_remaining_fuel` (has `Some(remaining_fuel)` for fuel-metered operations and `None` otherwise)
    Finished {
        values: Vec<Value>,
        maybe_remaining_fuel: Option<u64>,
    },
    /// represents a resumable that has ran out of fuel during execution, missing at least `required_fuel` units of fuel
    /// to continue further execution (this is None if unknown).
    Resumable {
        resumable: WasmResumable,
        required_fuel: Option<NonZeroU64>,
    },
    /// A host function was called by Wasm code. Use the [`HostCall`] to execute
    /// the host function and resume execution using the [`HostResumable`] and
    /// the return values produced by execution.
    HostCalled {
        host_call: HostCall,
        resumable: HostResumable,
    },
}
