use core::mem;

use alloc::{
    sync::{Arc, Weak},
    vec::Vec,
};

use crate::{
    core::slotmap::{SlotMap, SlotMapKey},
    execution::interpreter_loop,
    hooks::EmptyHookSet,
    rw_spinlock::RwSpinLock,
    value_stack::Stack,
    RuntimeError, Value,
};

use super::RuntimeInstance;

#[derive(Debug)]
pub struct Resumable {
    pub(crate) stack: Stack,
    pub(crate) pc: usize,
    pub(crate) stp: usize,
    pub(crate) current_func_addr: usize,
}

#[derive(Default)]
pub struct Dormitory(Arc<RwSpinLock<SlotMap<Resumable>>>);

impl Dormitory {
    #[allow(unused)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&self, resumable: Resumable) -> ResumableRef {
        let key = self.0.write().insert(resumable);

        ResumableRef {
            dormitory: Arc::downgrade(&self.0),
            key,
        }
    }
}

pub struct ResumableRef {
    dormitory: Weak<RwSpinLock<SlotMap<Resumable>>>,
    key: SlotMapKey<Resumable>,
}

impl ResumableRef {
    pub fn resume<T>(
        mut self,
        runtime_instance: &mut RuntimeInstance<T>,
        fuel: u32,
    ) -> Result<RunState, RuntimeError> {
        // Resuming requires `self`'s dormitory to still be alive
        let Some(dormitory) = self.dormitory.upgrade() else {
            return Err(RuntimeError::ResumableNotFound);
        };

        // Check the given `RuntimeInstance` is the same one used to create `self`
        if !Arc::ptr_eq(&dormitory, &runtime_instance.store.dormitory.0) {
            return Err(RuntimeError::ResumableNotFound);
        }

        // Obtain a write lock to the `Dormitory`
        let mut dormitory = dormitory.write();

        // TODO We might want to remove the `Resumable` here already and later reinsert it.
        // This would prevent holding the lock across the interpreter loop.
        let resumable = dormitory
            .get_mut(&self.key)
            .expect("the key to always be valid as self was not dropped yet");

        // Resume execution
        let result = interpreter_loop::run(
            resumable,
            &mut runtime_instance.store,
            EmptyHookSet,
            Some(fuel),
        );

        match result {
            Ok(()) => {
                let resumable = dormitory.remove(&self.key)
                    .expect("that the resumable could not have been removed already, because then this self could not exist");

                // Take the `Weak` pointing to the dormitory out of `self` and replace it with a default `Weak`.
                // This causes the `Drop` impl of `self` to directly quit preventing it from unnecessarily locking the dormitory.
                let _dormitory = mem::take(&mut self.dormitory);

                Ok(RunState::Finished(resumable.stack.into_values()))
            }
            Err(RuntimeError::OutOfFuel) => Ok(RunState::Resumable(self)),
            Err(err) => Err(err),
        }
    }
}

impl Drop for ResumableRef {
    fn drop(&mut self) {
        let Some(dormitory) = self.dormitory.upgrade() else {
            // Either the dormitory was already dropped or `self` was used to finish execution.
            return;
        };

        dormitory.write().remove(&self.key)
            .expect("that the resumable could not have been removed already, because then this self could not exist or the dormitory weak pointer would have been None");
    }
}

pub enum RunState {
    Finished(Vec<Value>),
    Resumable(ResumableRef),
}
