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
    RuntimeError, RuntimeInstance, Value,
};

#[derive(Debug)]
pub(super) struct Resumable {
    pub stack: Stack,
    pub pc: usize,
    pub stp: usize,
    pub current_func_addr: usize,
}

#[derive(Default)]
pub struct Dormitory(SlotMap<Resumable>);
pub struct ResumableAddr(SlotMapKey<Resumable>);

impl Dormitory {
    #[allow(unused)]
    pub(super) fn new() -> Dormitory {
        Self::default()
    }

    #[allow(unused)]
    pub(super) fn get(&self, resumable_addr: &ResumableAddr) -> Result<&Resumable, RuntimeError> {
        self.0
            .get(&resumable_addr.0)
            .ok_or(RuntimeError::ResumableNotFound)
    }

    pub(super) fn get_mut(
        &mut self,
        resumable_addr: &ResumableAddr,
    ) -> Result<&mut Resumable, RuntimeError> {
        self.0
            .get_mut(&resumable_addr.0)
            .ok_or(RuntimeError::ResumableNotFound)
    }

    pub(super) fn insert(&mut self, resumable: Resumable) -> ResumableAddr {
        ResumableAddr(self.0.insert(resumable))
    }

    pub(super) fn remove(
        &mut self,
        resumable_addr: &ResumableAddr,
    ) -> Result<Vec<Value>, RuntimeError> {
        let resumable = self
            .0
            .remove(&resumable_addr.0)
            .ok_or(RuntimeError::ResumableNotFound)?;
        Ok(resumable.stack.into_values())
    }
}

pub enum RunState {
    Finished(Vec<Value>),
    Resumable(ResumableRef),
}

pub struct ResumableRef {
    pub resumable_addr: ResumableAddr,
    pub dormitory: Weak<RwSpinLock<Dormitory>>,
}
impl ResumableRef {
    #[allow(unused)]
    pub fn resume<T>(
        self,
        runtime_instance: &mut RuntimeInstance<T>,
        fuel: u32,
    ) -> Result<RunState, RuntimeError> {
        let Some(dormitory) = self.dormitory.upgrade() else {
            return Err(RuntimeError::ResumableNotFound);
        };
        let mut dormitory = if Arc::ptr_eq(&dormitory, &runtime_instance.store.dormitory) {
            dormitory.write()
        } else {
            return Err(RuntimeError::ResumableNotFound);
        };
        let resumable = dormitory.get_mut(&self.resumable_addr)?;
        let result = interpreter_loop::run(
            resumable,
            &mut runtime_instance.store,
            EmptyHookSet,
            Some(fuel),
        );

        match result {
            Ok(_) => dormitory
                .remove(&self.resumable_addr)
                .map(RunState::Finished),
            Err(RuntimeError::OutOfFuel) => Ok(RunState::Resumable(self)),
            Err(err) => Err(err),
        }
    }
}

impl Drop for ResumableRef {
    fn drop(&mut self) {
        if let Some(dormitory) = self.dormitory.upgrade() {
            // an Err indicates this resumable was already dropped, which is fine
            dormitory.write().remove(&self.resumable_addr).unwrap();
        }
    }
}
