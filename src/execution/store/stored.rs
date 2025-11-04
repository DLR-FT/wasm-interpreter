//! TODO: Logic that makes sure that objects, which belong to a specific [`Store`], are only used with that [`Store`].

use core::sync::atomic::{AtomicU64, Ordering};

use crate::{config::Config, core::reader::types::FuncType, RuntimeError};

use super::{addrs::FuncAddr, Store};

/// A unique identifier for a specfic [`Store]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct StoreId(u64);

impl StoreId {
    /// Creates a new unique [`StoreId`]
    #[allow(clippy::new_without_default)] // reason = "StoreId::default() might be misunderstood to be some default value. However, a
                                          // default value does not exist in that sense because every newly created StoreId must be unique"
    pub fn new() -> Self {
        static NEXT_STORE_ID: AtomicU64 = AtomicU64::new(0);

        // TODO find a fix for the default wrapping behaviour of `fetch_add`.
        // Maybe we could return a `RuntimeError` here?
        Self(NEXT_STORE_ID.fetch_add(1, Ordering::SeqCst))
    }
}

pub struct Stored<T> {
    id: StoreId,
    inner: T,
}

impl<T: Config> Store<'_, T> {
    pub fn wrap_stored<U>(&self, inner: U) -> Stored<U> {
        Stored { id: self.id, inner }
    }

    pub fn try_unwrap_stored<U>(&self, stored: Stored<U>) -> Result<U, RuntimeError> {
        if self.id == stored.id {
            Ok(stored.inner)
        } else {
            Err(RuntimeError::StoreIdMismatch)
        }
    }

    /// Gets the type of a function by its addr.
    ///
    /// See: WebAssembly Specification 2.0 - 7.1.7 - func_type
    pub fn func_type(&self, func_addr: Stored<FuncAddr>) -> Result<FuncType, RuntimeError> {
        let func_addr = self.try_unwrap_stored(func_addr)?;
        Ok(self.func_type_unchecked(func_addr))
    }
}
