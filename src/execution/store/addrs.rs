//! Type definitions for addr types
//!
//! An addr (short for: address) is a dynamic index only known at runtime into a store.
//! There are addr types for different index spaces, such as memories [`MemAddr`], globals [`GlobalAddr`] or functions [`FuncAddr`].

use super::{FuncInst, Store};

/// Represents the address of a function within a WebAssembly module.
// TODO this was copied over from the previous FuncAddr
//   Functions in WebAssembly modules can be either:
//   - **Defined**: Declared and implemented within the module.
//   - **Imported**: Declared in the module but implemented externally.
//
//   [`FuncAddr`] provides a unified representation for both types. Internally,
//   the address corresponds to an index in a combined function namespace,
//   typically represented as a vector.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct FuncAddr(usize);

impl FuncAddr {
    /// This is unfortunately needed as a default value for the base [`CallFrame`] in every call stack.
    pub const INVALID: Self = FuncAddr(usize::MAX);

    pub fn into_inner(self) -> usize {
        self.0
    }
}

// TODO choose one of: `get_function`, `get_function_associated_fn`
impl<T> Store<'_, T> {
    pub(crate) fn get_function(&self, addr: FuncAddr) -> &FuncInst<T> {
        self.functions
            .get(addr.0)
            .expect("func addrs to always be valid if the correct store is used")
    }

    /// This allows callers to partially borrow only the `functions` field of a [`Store`] when accessing a function instance.
    pub(crate) fn get_function_associated_fn(
        functions: &[FuncInst<T>],
        addr: FuncAddr,
    ) -> &FuncInst<T> {
        functions
            .get(addr.0)
            .expect("func addrs to always be valid if the correct store is used")
    }

    pub(crate) fn push_func_inst(&mut self, instance: FuncInst<T>) -> FuncAddr {
        let new_addr = self.functions.len();
        self.functions.push(instance);
        FuncAddr(new_addr)
    }
}
