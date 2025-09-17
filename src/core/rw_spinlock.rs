//! Naive implementation of spin based locking mechanisms
//!
//! This module provides implementations for locking mechanisms required.
//!
//! # Acknowledgement
//!
//! This implementation is largely inspired by the book
//! ["Rust Atomics and Locks" by Mara Bos](https://marabos.nl/atomics/).

use core::cell::UnsafeCell;
use core::hint::{self};
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicU32, Ordering};

/// A spinlock based, read-write lock which favours writers over readers
///
/// # Properties
///
/// - Read-write semantics allow for multiple readers at the same time, but require exclusive access
///   for a writer
/// - Spin based, e.g. waiting for the lock wastes CPU cycles in a busy loop
///    - This design however enables an implementation independent of operating system features like
///      condition variables, the only requirement are 32 bit atomics in the ISA
/// - Biased towards writes: once a writer waits for the lock, all new reader wait until that writer
///   got access
pub struct RwSpinLock<T> {
    /// The inner data protected by this lock
    inner: UnsafeCell<T>,

    /// Lock state (on ambiguity, the state closer to the top takes precedence)
    ///
    /// - `0` means there are no readers nor any writer
    /// - `u32::MAX` means there is a single active writer
    /// - `state % 2 == 0` means there are `state / 2` active readers
    /// - `state % 2 != 0` means there are `(state - 1) / 2` active readers and at least one waiting
    ///   writer
    state: AtomicU32,
}

impl<T> RwSpinLock<T> {
    /// Create a new instance of self, wrapping the `value` of type `T`
    pub fn new(value: T) -> Self {
        Self {
            inner: UnsafeCell::new(value),
            state: AtomicU32::new(0),
        }
    }

    // Get read access to the value wrapped in this [`RwSpinLock`]
    pub fn read(&self) -> ReadLockGuard<'_, T> {
        // get the current state
        let mut s = self.state.load(Ordering::Relaxed); // ordering by the book

        loop {
            // s is even, so there are maybe active readers but no active or waiting writer
            // -> reader can acquire read guard (as long as an overflow is avoided)
            if s % 2 == 0 && s < u32::MAX - 2 {
                match self.state.compare_exchange_weak(
                    s,
                    s + 2,
                    Ordering::Acquire,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => return ReadLockGuard { lock: self },
                    Err(update_s) => s = update_s,
                }
            }

            // there is one active (`s == u32::MAX`) or at least one waiting (otherwise) writer
            // -> spin, re-load s and try again
            if s % 2 == 1 {
                hint::spin_loop();
                s = self.state.load(Ordering::Relaxed); // ordering by the book
            }
        }
    }

    // Get write access to the value wrapped in this [`RwSpinLock`]
    pub fn write(&self) -> WriteLockGuard<'_, T> {
        let mut s = self.state.load(Ordering::Relaxed);

        loop {
            // there is no active reader (`s >= 2 && s % 2 == 0`) or writer (`s == u32::MAX`)
            if s <= 1 {
                match self
                    .state
                    .compare_exchange(s, u32::MAX, Ordering::Acquire, Ordering::Relaxed)
                {
                    Ok(_) => return WriteLockGuard { lock: self },
                    Err(updated_s) => {
                        s = updated_s;
                        continue;
                    }
                }
            }

            // announce that a writer is waiting if this is not yet announced
            if s % 2 == 0 {
                match self
                    .state
                    // ordering by the book
                    .compare_exchange(s, s + 1, Ordering::Relaxed, Ordering::Relaxed)
                {
                    Ok(_) => {}
                    Err(updated_s) => {
                        s = updated_s;
                        continue;
                    }
                }
            }

            // wait was announced, there are still active readers
            // -> spin, re-load s, continue from the start of the lop
            hint::spin_loop();
            s = self.state.load(Ordering::Relaxed);
        }
    }
}

// SAFETY: When the inner `T` is `Sync`, the `RwSpinlock<T>` can be `Sync` as well
unsafe impl<T> Sync for RwSpinLock<T> where T: Send + Sync {}

/// Read guard for the [`RwSpinLock`]
pub struct ReadLockGuard<'a, T> {
    lock: &'a RwSpinLock<T>,
}

impl<T> Deref for ReadLockGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY: For as long as a `ReadLockGuard` exists, it can dereference to a shared reference
        // to the inner data can be handed out
        unsafe { &*self.lock.inner.get() }
    }
}

impl<T> Drop for ReadLockGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.state.fetch_sub(2, Ordering::Release); // ordering by the book
    }
}

/// Write guard for the [`RwSpinLock`]
pub struct WriteLockGuard<'a, T> {
    lock: &'a RwSpinLock<T>,
}

impl<T> Deref for WriteLockGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY: For as long as a `WriteLockGuard` exists, it can derefence to a shared reference
        // to the inner data
        unsafe { &*self.lock.inner.get() }
    }
}

impl<T> DerefMut for WriteLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: For as long as a `WriteLockGuard` exists, it can derefence to a mutable
        // references to the inner data
        unsafe { &mut *self.lock.inner.get() }
    }
}

impl<T> Drop for WriteLockGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.state.store(0, Ordering::Release); // ordering by the book
    }
}
