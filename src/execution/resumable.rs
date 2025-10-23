use core::num::NonZeroU32;

use alloc::{
    sync::{Arc, Weak},
    vec::Vec,
};

use crate::{
    addrs::FuncAddr,
    core::slotmap::{SlotMap, SlotMapKey},
    rw_spinlock::RwSpinLock,
    value_stack::Stack,
    Value,
};

#[derive(Debug)]
pub(crate) struct Resumable {
    pub(crate) stack: Stack,
    pub(crate) pc: usize,
    pub(crate) stp: usize,
    pub(crate) current_func_addr: FuncAddr,
    pub(crate) maybe_fuel: Option<u32>,
}

#[derive(Default)]
pub(crate) struct Dormitory(pub(crate) Arc<RwSpinLock<SlotMap<Resumable>>>);

impl Dormitory {
    #[allow(unused)]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn insert(&self, resumable: Resumable) -> InvokedResumableRef {
        let key = self.0.write().insert(resumable);

        InvokedResumableRef {
            dormitory: Arc::downgrade(&self.0),
            key,
        }
    }
}

pub struct InvokedResumableRef {
    pub(crate) dormitory: Weak<RwSpinLock<SlotMap<Resumable>>>,
    pub(crate) key: SlotMapKey<Resumable>,
}

pub struct FreshResumableRef {
    pub(crate) func_addr: FuncAddr,
    pub(crate) params: Vec<Value>,
    pub(crate) maybe_fuel: Option<u32>,
}

/// An object associated to a resumable that is held internally. The variant `ResumableRef::Fresh` indicates this
/// resumable has never been invoked/resumed to. `ResumableRef::Invoked` indicates this resumable has been
/// invoked/resumed to at least once.
pub enum ResumableRef {
    Fresh(FreshResumableRef),
    Invoked(InvokedResumableRef),
}

impl Drop for InvokedResumableRef {
    fn drop(&mut self) {
        let Some(dormitory) = self.dormitory.upgrade() else {
            // Either the dormitory was already dropped or `self` was used to finish execution.
            return;
        };

        dormitory.write().remove(&self.key)
            .expect("that the resumable could not have been removed already, because then this self could not exist or the dormitory weak pointer would have been None");
    }
}

/// Represents the state of a possibly interrupted resumable. `RunState::Finished` represents a resumable that has
/// executed completely. `RunState::Resumable` represents a resumable that has ran out of fuel during execution, missing
/// at least `required_fuel` units of fuel to continue further execution.
pub enum RunState {
    Finished(Vec<Value>),
    Resumable {
        resumable_ref: ResumableRef,
        required_fuel: NonZeroU32,
    },
}

#[cfg(test)]
mod test {
    use crate::{addrs::FuncAddr, value_stack::Stack};

    use super::{Dormitory, Resumable};

    /// Test that a dormitory can be constructed and that a resumable can be inserted
    #[test]
    fn dormitory_constructor() {
        let dorm = Dormitory::new();

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
