use core::{mem, num::NonZeroU32};

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
    pub(crate) maybe_fuel: Option<u32>,
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
    /// calls its argument `f` with a mutable reference of the fuel of the respective [`ResumableRef`].
    ///
    /// Fuel is stored as an [`Option<u32>`], where `None` means that fuel is disabled and `Some(x)` means that `x` units of fuel is left.
    /// A ubiquitious use of this method would be using `f` to read or mutate the current fuel amount of the respective [`ResumableRef`].
    /// # Example
    /// ```
    /// use wasm::{resumable::RunState, validate, RuntimeInstance};
    /// let wasm = [ 0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
    ///             0x01, 0x04, 0x01, 0x60, 0x00, 0x00, 0x03, 0x02,
    ///             0x01, 0x00, 0x07, 0x09, 0x01, 0x05, 0x6c, 0x6f,
    ///             0x6f, 0x70, 0x73, 0x00, 0x00, 0x0a, 0x09, 0x01,
    ///             0x07, 0x00, 0x03, 0x40, 0x0c, 0x00, 0x0b, 0x0b ];
    /// // a simple module with a single function looping forever
    /// let mut instance = RuntimeInstance::new_named((), "module", &validate(&wasm).unwrap()).unwrap();
    /// let func_ref = instance.get_function_by_name("module", "loops").unwrap();
    /// let resumable = instance.invoke_resumable(&func_ref, vec![], 0).unwrap();
    /// match resumable {
    ///     RunState::Resumable { resumable_ref, .. } => {
    ///         // inspect and modify fuel content
    ///         resumable_ref.access_fuel_mut(&mut instance, |x| { assert_eq!(*x, Some(0)); *x = None; }).unwrap();
    ///     }
    ///     _ => unreachable!("this function loops forever")
    /// }
    /// ```
    pub fn access_fuel_mut<T, R>(
        &self,
        runtime_instance: &mut RuntimeInstance<T>,
        f: impl FnOnce(&mut Option<u32>) -> R,
    ) -> Result<R, RuntimeError> {
        // Resuming requires `self`'s dormitory to still be alive
        let Some(dormitory) = self.dormitory.upgrade() else {
            return Err(RuntimeError::ResumableNotFound);
        };

        // Check the given `RuntimeInstance` is the same one used to create `self`
        if !Arc::ptr_eq(&dormitory, &runtime_instance.store.dormitory.0) {
            return Err(RuntimeError::ResumableNotFound);
        }

        let mut dormitory = dormitory.write();

        let resumable = dormitory
            .get_mut(&self.key)
            .expect("the key to always be valid as self was not dropped yet");

        Ok(f(&mut resumable.maybe_fuel))
    }

    /// resumes execution of the resumable.
    pub fn resume<T>(
        mut self,
        runtime_instance: &mut RuntimeInstance<T>,
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
        let result = interpreter_loop::run(resumable, &mut runtime_instance.store, EmptyHookSet)?;

        match result {
            None => {
                let resumable = dormitory.remove(&self.key)
                    .expect("that the resumable could not have been removed already, because then this self could not exist");

                // Take the `Weak` pointing to the dormitory out of `self` and replace it with a default `Weak`.
                // This causes the `Drop` impl of `self` to directly quit preventing it from unnecessarily locking the dormitory.
                let _dormitory = mem::take(&mut self.dormitory);

                Ok(RunState::Finished(resumable.stack.into_values()))
            }
            Some(required_fuel) => Ok(RunState::Resumable {
                resumable_ref: self,
                required_fuel,
            }),
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
    Resumable {
        resumable_ref: ResumableRef,
        required_fuel: NonZeroU32,
    },
}

#[cfg(test)]
mod test {
    use crate::value_stack::Stack;

    use super::{Dormitory, Resumable};

    /// Test that a dormitory can be constructed and that a resumable can be inserted
    #[test]
    fn dormitory_constructor() {
        let dorm = Dormitory::new();

        let resumable = Resumable {
            stack: Stack::new(),
            pc: 11,
            stp: 13,
            current_func_addr: 17,
            maybe_fuel: None,
        };

        dorm.insert(resumable);
    }
}
