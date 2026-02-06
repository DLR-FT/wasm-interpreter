use core::num::NonZeroU64;

use alloc::vec::Vec;

use crate::{
    addrs::FuncAddr,
    core::slotmap::{SlotMap, SlotMapKey},
    value_stack::Stack,
    Value,
};

#[derive(Debug)]
pub(crate) struct Resumable {
    pub(crate) stack: Stack,
    pub(crate) pc: usize,
    pub(crate) stp: usize,
    pub(crate) current_func_addr: FuncAddr,
    pub(crate) maybe_fuel: Option<u64>,
}

#[derive(Default)]
pub(crate) struct Dormitory(SlotMap<Resumable>);

impl Dormitory {
    #[allow(unused)]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn insert(&mut self, resumable: Resumable) -> InvokedResumableRef {
        let key = self.0.insert(resumable);
        InvokedResumableRef { key }
    }

    pub(crate) fn get_mut(
        &mut self,
        resumable_ref: &InvokedResumableRef,
    ) -> Option<&mut Resumable> {
        self.0.get_mut(&resumable_ref.key)
    }

    pub(crate) fn remove(&mut self, resumable_ref: InvokedResumableRef) -> Option<Resumable> {
        self.0.remove(&resumable_ref.key)
    }
}

/// # Note
///
/// Dropping a resumable ref does not deallocate the resumable anymore. It is up
/// to the user to implement such garbage collection algorithms by using
/// [`Store::drop_resumable`] to prevent memory leaks.
pub struct InvokedResumableRef {
    pub(crate) key: SlotMapKey<Resumable>,
}

pub struct FreshResumableRef {
    pub(crate) func_addr: FuncAddr,
    pub(crate) params: Vec<Value>,
    pub(crate) maybe_fuel: Option<u64>,
}

/// An object associated to a resumable that is held internally.
pub enum ResumableRef {
    /// indicates this resumable has never been invoked/resumed to.
    Fresh(FreshResumableRef),
    /// indicates this resumable has been invoked/resumed to at least once.
    Invoked(InvokedResumableRef),
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
        resumable_ref: ResumableRef,
        required_fuel: NonZeroU64,
    },
}

#[cfg(test)]
mod test {
    use crate::{addrs::FuncAddr, value_stack::Stack};

    use super::{Dormitory, Resumable};

    /// Test that a dormitory can be constructed and that a resumable can be inserted
    #[test]
    fn dormitory_constructor() {
        let mut dorm = Dormitory::new();

        let resumable = Resumable {
            stack: Stack::new(),
            pc: 11,
            stp: 13,
            current_func_addr: FuncAddr::INVALID,
            maybe_fuel: None,
        };

        dorm.insert(resumable);
    }
}
