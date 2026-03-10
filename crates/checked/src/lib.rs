//! Definitions for checked, safe variants of methods defined on [`Store`](wasm::Store) and
//! [`Linker`](wasm::linker::Linker).
//!
//! This module defines extensions in the form of new types and new methods. It
//! only relies on the fact that the [`Store`](wasm::Store) and the
//! [`Linker`](wasm::linker::Linker) both store a [`StoreId`]. No other changes
//! are required to be made to the main interpreter for this checked API.
//!
//!
//! All extension methods defined in this module use special _stored_ objects.
//! These objects are essentially normal objects like
//! [`FuncAddr`](wasm::addrs::FuncAddr), [`RunState`](wasm::resumable::RunState)
//! or [`Value`](wasm::Value). However, they also contain an additional field of
//! type [`StoreId`] as a tag to know to which [`Store`](wasm::Store) they
//! belong to.
//!
//! While this is easy for address types like
//! [`FuncAddr`](wasm::addrs::FuncAddr) or [`MemAddr`](wasm::addrs::MemAddr),
//! some types are enums and their variants are visible to the user. For
//! example, consider the [`Value`](wasm::Value) enum, where users have full
//! access to all of its variants. To be able to attach a tag only to the
//! [`Value::Ref`](wasm::Value::Ref) variant of this enum, the entire enum has
//! to be re-defined. The result is a completely new type [`StoredValue`].

#![no_std]

extern crate alloc;

use core::sync::atomic::{AtomicU64, Ordering};

use alloc::vec::Vec;
use wasm::RuntimeError;

mod interop;
mod linker;
mod store;
mod stored_types;
mod value;

pub use interop::*;
pub use linker::*;
pub use store::*;
pub use stored_types::*;
pub use value::*;

/// A unique identifier for a specific [`Store`]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct StoreId(u64);

impl StoreId {
    /// Creates a new unique [`StoreId`]
    #[allow(clippy::new_without_default)] // reason = "StoreId::default() might be misunderstood to be some
                                          // default value. However, a default value does not exist in that
                                          // sense because every newly created StoreId must be unique. Also
                                          // we don't want to allow the user to create new instances of
                                          // this object."
    pub(crate) fn new() -> Self {
        static NEXT_STORE_ID: AtomicU64 = AtomicU64::new(0);

        // TODO find a fix for the default wrapping behavior of `fetch_add`.
        // Maybe we could return a `RuntimeError` here?
        Self(NEXT_STORE_ID.fetch_add(1, Ordering::SeqCst))
    }
}

/// A trait for types that might have a [`StoreId`] attached to them, so-called
/// _stored_ types.
pub trait AbstractStored: Sized {
    type BareTy: Sized;

    /// Creates a new stored object
    ///
    /// # Safety
    ///
    /// The caller has to guarantee that the bare value comes from a [`Store`]
    /// with the given [`StoreId`].
    unsafe fn from_bare(bare_value: Self::BareTy, id: StoreId) -> Self;

    /// Gets the id of this stored object
    ///
    /// Not all stored objects require to have an id attached to them.
    fn id(&self) -> Option<StoreId>;

    /// Converts this stored object into its bare form that does not have any [`StoreId`] attached to it.
    fn into_bare(self) -> Self::BareTy;

    /// Checks if this stored object comes from a specific store by its
    /// [`StoreId`]. If true, it converts self into its bare form, otherwise an
    /// error is returned.
    ///
    /// # Errors
    ///
    /// - [`RuntimeError::StoreIdMismatch`]
    fn try_unwrap_into_bare(
        self,
        expected_store_id: StoreId,
    ) -> Result<Self::BareTy, RuntimeError> {
        if let Some(id) = self.id() {
            if id != expected_store_id {
                return Err(RuntimeError::StoreIdMismatch);
            }
        }

        Ok(self.into_bare())
    }
}

impl<T: AbstractStored> AbstractStored for Vec<T> {
    type BareTy = Vec<T::BareTy>;

    /// Creates a new vector of stored objects from a vector of non-stored
    /// objects.
    ///
    /// # Safety
    ///
    /// The caller has to guarantee that all bare values in the given vector
    /// come from a single [`Store`](crate::store::Store) with the given
    /// [`StoreId`].
    unsafe fn from_bare(bare_value: Self::BareTy, id: StoreId) -> Self {
        bare_value
            .into_iter()
            .map(|bare| {
                // SAFETY: Upheld by caller
                unsafe { T::from_bare(bare, id) }
            })
            .collect()
    }

    fn id(&self) -> Option<StoreId> {
        self.iter().find_map(T::id)
    }

    fn into_bare(self) -> Self::BareTy {
        self.into_iter().map(T::into_bare).collect()
    }
}
