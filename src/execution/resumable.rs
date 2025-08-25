use alloc::{collections::btree_map::BTreeMap, vec::Vec};

use crate::{
    execution::interpreter_loop, hooks::EmptyHookSet, value_stack::Stack, RuntimeError,
    RuntimeInstance, Value,
};

#[derive(Debug)]
pub(super) struct Resumable {
    pub stack: Stack,
    pub pc: usize,
    pub stp: usize,
    pub current_func_addr: usize,
}

#[derive(Default, Debug)]
pub struct Dormitory {
    last_resumable_addr: usize,
    resumables: BTreeMap<usize, Resumable>,
}

impl Dormitory {
    #[allow(unused)]
    pub(super) fn get(&self, resumable_addr: usize) -> Result<&Resumable, RuntimeError> {
        self.resumables
            .get(&resumable_addr)
            .ok_or(RuntimeError::ResumableNotFound)
    }

    pub(super) fn get_mut(
        &mut self,
        resumable_addr: usize,
    ) -> Result<&mut Resumable, RuntimeError> {
        self.resumables
            .get_mut(&resumable_addr)
            .ok_or(RuntimeError::ResumableNotFound)
    }

    pub(super) fn insert(&mut self, resumable: Resumable) -> usize {
        self.last_resumable_addr += 1;
        let None = self
            .resumables
            .insert(self.last_resumable_addr - 1, resumable)
        else {
            unreachable!("resumable addresses do not repeat")
        };
        self.last_resumable_addr - 1
    }

    pub(super) fn remove(&mut self, resumable_addr: usize) -> Result<Vec<Value>, RuntimeError> {
        let resumable = self
            .resumables
            .remove(&resumable_addr)
            .ok_or(RuntimeError::ResumableNotFound)?;
        Ok(resumable.stack.into_values())
    }
}

pub enum RunState {
    Finished(Vec<Value>),
    Resumable(ResumableRef),
}

pub struct ResumableRef(pub(super) usize);
impl ResumableRef {
    #[allow(unused)]
    pub fn resume<T>(
        self,
        runtime_instance: &mut RuntimeInstance<T>,
        fuel: u32,
    ) -> Result<RunState, RuntimeError> {
        // TODO fix error
        let result = interpreter_loop::run(
            self.0,
            &mut runtime_instance.store,
            EmptyHookSet,
            Some(fuel),
        )?;
        if result != usize::MAX {
            Ok(RunState::Resumable(self))
        } else {
            runtime_instance
                .store
                .dormitory
                .remove(self.0)
                .map(RunState::Finished)
        }
    }
}
