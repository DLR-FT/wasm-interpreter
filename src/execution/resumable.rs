use core::num::NonZeroU64;

use alloc::vec::Vec;

use crate::{addrs::FuncAddr, value_stack::Stack, HaltExecutionError, Value};

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

#[derive(Debug)]
pub struct HostResumable<T> {
    /// Must be a host function instance.
    pub(crate) func_addr: FuncAddr,
    /// Must contain the correct types as specified by the [`FuncType`](crate::FuncType) for
    /// `func_addr`.
    pub(crate) params: Vec<Value>,
    pub(crate) hostcode: fn(&mut T, Vec<Value>) -> Result<Vec<Value>, HaltExecutionError>,
}

#[derive(Debug)]
pub enum Resumable<T> {
    Wasm(WasmResumable),
    Host(HostResumable<T>),
}

impl<T> Resumable<T> {
    /// Tries to convert this [`Resumable`] into a [`WasmResumable`]
    pub fn as_wasm_resumable(self) -> Option<WasmResumable> {
        match self {
            Resumable::Wasm(wasm_resumable) => Some(wasm_resumable),
            Resumable::Host(_) => None,
        }
    }

    /// Tries to convert this [`Resumable`] into a [`HostResumable`]
    pub fn as_host_resumable(self) -> Option<HostResumable<T>> {
        match self {
            Resumable::Wasm(_) => None,
            Resumable::Host(host_resumable) => Some(host_resumable),
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
    /// to continue further execution.
    Resumable {
        resumable: WasmResumable,
        required_fuel: NonZeroU64,
    },
}
