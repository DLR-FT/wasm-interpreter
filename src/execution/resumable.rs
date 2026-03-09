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

#[derive(Debug)]
pub struct HostResumable<T> {
    /// Must be a host function instance.
    pub(crate) func_addr: FuncAddr,
    /// Must contain the correct types as specified by the [`FuncType`](crate::FuncType) for
    /// `func_addr`.
    pub(crate) params: Vec<Value>,
    pub(crate) hostcode: fn(&mut T, Vec<Value>) -> Result<Vec<Value>, HaltExecutionError>,
    pub(crate) maybe_fuel: Option<u64>,
}

#[derive(Debug)]
pub enum Resumable<T> {
    Wasm(WasmResumable),
    Host(HostResumable<T>),
}

impl<T> Resumable<T> {
    pub fn fuel(&self) -> Option<u64> {
        match self {
            Resumable::Wasm(wasm_resumable) => wasm_resumable.maybe_fuel,
            Resumable::Host(host_resumable) => host_resumable.maybe_fuel,
        }
    }

    pub fn fuel_mut(&mut self) -> &mut Option<u64> {
        match self {
            Resumable::Wasm(wasm_resumable) => &mut wasm_resumable.maybe_fuel,
            Resumable::Host(host_resumable) => &mut host_resumable.maybe_fuel,
        }
    }
}

/// Represents the state of a possibly interrupted resumable.
pub enum RunState<T> {
    /// represents a resumable that has executed completely with return values `values` and possibly remaining fuel
    /// `maybe_remaining_fuel` (has `Some(remaining_fuel)` for fuel-metered operations and `None` otherwise)
    Finished {
        values: Vec<Value>,
        maybe_remaining_fuel: Option<u64>,
    },
    /// represents a resumable that has ran out of fuel during execution, missing at least `required_fuel` units of fuel
    /// to continue further execution.
    Resumable {
        resumable: Resumable<T>, // TODO make this a `WasmResumable`
        required_fuel: NonZeroU64,
    },
}
