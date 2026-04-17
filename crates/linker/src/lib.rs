//! A Name Resolution Based Linker

#![no_std]
#![deny(
    clippy::missing_safety_doc,
    clippy::undocumented_unsafe_blocks,
    unsafe_op_in_unsafe_fn
)]

extern crate alloc;

use alloc::{
    borrow::ToOwned,
    collections::btree_map::{BTreeMap, Entry},
    string::String,
    vec::Vec,
};

use wasm::{
    addrs::ModuleAddr, config::Config, store::InstantiationOutcome,
    validation_config::ValidationConfig, ExternVal, RuntimeError, Store, ValidationInfo,
};

/// A linker used to link a module's imports against extern values previously
/// defined in this [`Linker`] context.
///
/// # Manual Instantiation vs. Instantiation through [`Linker`]
///
/// Traditionally, module instances are instantiated via the method
/// [`Store::module_instantiate`], which is part of the official Embedder API
/// defined by the specification. However, this method accepts a list of extern
/// values as an argument. Therefore, if the user wants to manually perform
/// linking they have to figure out the imports of their module, then gather the
/// correct extern values, and finally call the instantiation method.
///
/// This process of manual linking is very tedious and error-prone, which is why
/// the [`Linker`] exists. It builds on top of the original instantiation method
/// with [`Linker::module_instantiate`]. Internally this method performs name
/// resolution and then calls the original instantiation. Name resolution is
/// performed on all extern values which were previously defined in the current
/// context.
///
/// # Extern values
///
/// An extern value is represented as a [`ExternVal`]. It contains an address to
/// some store-allocated instance. In a linker context, every external value is
/// stored in map with a unique key `(module name, name)`. To define new extern
/// value in some linker context, use [`Linker::define`] or
/// [`Linker::define_module_instance`].
///
/// # Relationship with [`Store`]
///
/// There is a N-to-1 relationship between the [`Linker`] and the [`Store`].
/// This means that multiple linkers can be used with the same store, while
/// every linker may be used only with one specific store.
///
/// Due to performance reasons, this bookkeeping is not done by the [`Linker`]
/// itself. Instead it is the user's responsibility to uphold this requirement.
#[derive(Clone, Default)]
pub struct Linker {
    /// All extern values in the current linker context by their import keys.
    ///
    /// It is guaranteed that the addresses of all extern values belong to the
    /// same [`Store`].
    extern_vals: BTreeMap<ImportKey, ExternVal>,
}

impl Linker {
    /// Creates a new [`Linker`] that is not yet associated to any specific [`Store`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Defines a new extern value in the current [`Linker`] context.
    ///
    /// # Safety
    ///
    /// It must be made sure that this [`Linker`] is only used with one specific
    /// [`Store`] and addresses that belong to that store.
    pub unsafe fn define(
        &mut self,
        module_name: String,
        name: String,
        extern_val: ExternVal,
    ) -> Result<(), RuntimeError> {
        match self.extern_vals.entry(ImportKey { module_name, name }) {
            Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(extern_val);
                Ok(())
            }
            Entry::Occupied(_occupied_entry) => Err(RuntimeError::DuplicateExternDefinition),
        }
    }

    /// Defines all exports of some module instance as extern values in the
    /// current [`Linker`].
    ///
    /// # Safety
    ///
    /// It must be guaranteed that this [`Linker`] is only ever used with one
    /// specific [`Store`] and that the given [`ModuleAddr`] is valid in this
    /// store.
    pub unsafe fn define_module_instance<T: Config>(
        &mut self,
        store: &Store<T>,
        module_name: String,
        module: ModuleAddr,
    ) -> Result<(), RuntimeError> {
        // SAFETY: The caller ensures that the given module address is valid in
        // the given store.
        let module_exports = unsafe { store.instance_exports(module) };
        for export in module_exports {
            // SAFETY: The module and thus also its exported extern values come
            // from the same store used now. Therefore, the extern values must
            // be valid in this store.
            unsafe { self.define(module_name.clone(), export.0, export.1)? };
        }

        Ok(())
    }

    /// Tries to get some extern value by its module name and name.
    ///
    /// It is guaranteed that the address contained by the returned
    /// [`ExternVal`] is part of the [`Store`] used with this [`Linker`].
    pub fn get(&self, module_name: String, name: String) -> Option<ExternVal> {
        self.extern_vals
            .get(&ImportKey { module_name, name })
            .copied()
    }

    /// Performs initial linking of a [`ValidationInfo`]'s imports producing a
    /// list of extern values usable with [`Store::module_instantiate`].
    ///
    /// # A note on type checking
    ///
    /// This method does not perform type checking on the extern values.
    /// Therefore, using the returned list of extern values may still fail when
    /// trying to instantiate a module with it.
    // TODO find a better name for this method? Maybe something like `link`?
    pub fn instantiate_pre<T: ValidationConfig>(
        &self,
        validation_info: &ValidationInfo<T>,
    ) -> Option<Vec<ExternVal>> {
        validation_info
            .imports()
            .map(|(module_name, name, _desc)| self.get(module_name.to_owned(), name.to_owned()))
            .collect()
    }

    /// Variant of [`Store::module_instantiate`] with automatic name resolution
    /// in the current [`Linker`] context. Returns `None` if name resolution
    /// failed.
    ///
    /// # Safety
    ///
    /// It must be guaranteed that this [`Linker`] is only ever used with one
    /// specific [`Store`].
    pub unsafe fn module_instantiate<'b, T: Config, T2: ValidationConfig>(
        &self,
        store: &mut Store<'b, T>,
        validation_info: &ValidationInfo<'b, T2>,
        maybe_fuel: Option<u64>,
    ) -> Option<Result<InstantiationOutcome, RuntimeError>> {
        self.instantiate_pre(validation_info).map(|instantiate_pre|
            // SAFETY: Because all extern values in a single linker can only come
            // from one specific store, the current store must be the same store
            // used to define all previous extern values. Therefore, the extern
            // values in `instantiate_pre` must be from the same store that is
            // passed now. Thus, using them as imports for module instantiation is
            // sound.
            unsafe { store.module_instantiate(validation_info, instantiate_pre, maybe_fuel) })
    }
}

/// A key used by Wasm modules to identify the names of imports.
///
/// It consists of a module name and the name of the imported item itself.
#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
struct ImportKey {
    module_name: String,
    name: String,
}
