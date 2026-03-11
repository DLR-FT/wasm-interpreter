use alloc::{string::String, vec::Vec};
use wasm::{RuntimeError, ValidationInfo, addrs::ModuleAddr, config::Config};

use crate::{
    AbstractStored, StoreId,
    store::Store,
    stored_types::{Stored, StoredExternVal, StoredInstantiationOutcome},
};

#[derive(Default)]
pub struct Linker {
    inner: wasm::linker::Linker,

    /// This is for the checked API which makes sure that all objects used
    /// originate from the same [`Store`].
    ///
    /// Initially the store id is `None`. Only when a store-specific object or a
    /// [`Store`] itself is used with a checked method, is this field set.  Once
    /// initialized it is never written to again.
    store_id: Option<StoreId>,
}

// All functions in this impl block must occur in the same order as they are
// defined in for the unchecked [`Linker`] methods. Also all functions must
// follow the same implementation scheme to make sure they are only light
// wrappers:
//
// 1. get or insert the `StoreId` [of the store associated with the current `Linker`]
// 2. try unwrap [stored parameter objects]
// 3. call [unchecked method]
// 4. rewrap [results into stored objects]
// 5. return [stored result objects]
impl Linker {
    /// Creates a new [`Linker`] that is not yet associated to any specific [`Store`].
    pub fn new() -> Self {
        Self::default()
    }

    /// This is a safe variant of [`Linker::define_unchecked`](crate::linker::Linker::define_unchecked).
    pub fn define(
        &mut self,
        module_name: String,
        name: String,
        extern_val: StoredExternVal,
    ) -> Result<(), RuntimeError> {
        // 1. get or insert the `StoreId`
        let extern_val_store_id = extern_val
            .id()
            .expect("this type to always contain a StoreId");
        let linker_store_id = *self.store_id.get_or_insert(extern_val_store_id);
        if linker_store_id != extern_val_store_id {
            panic!("Store id mismatch");
        }
        // 2. try unwrap
        let extern_val = extern_val.try_unwrap_into_bare(linker_store_id);
        // 3. call
        // SAFETY: It was just checked that the `ExternVal` came from the store
        // with the same id that is cached in the current linker instance.
        unsafe { self.inner.define_unchecked(module_name, name, extern_val) }?;
        // 4. rewrap
        // result is the unit type.
        // 5. return
        Ok(())
    }

    /// This is a safe variant of [`Linker::define_module_instance_unchecked`](crate::linker::Linker::define_module_instance_unchecked).
    pub fn define_module_instance<T: Config>(
        &mut self,
        store: &Store<T>,
        module_name: String,
        module: Stored<ModuleAddr>,
    ) -> Result<(), RuntimeError> {
        // 1. get or insert the `StoreId`
        let module_store_id = module.id().expect("this type to always contain a StoreId");
        let linker_store_id = *self.store_id.get_or_insert(module_store_id);
        if linker_store_id != module_store_id {
            panic!("Store id mismatch");
        }
        // 2. try unwrap
        let module = module.try_unwrap_into_bare(linker_store_id);
        // 3. call
        // SAFETY: It was just checked that the `ExternVal` came from the store
        // with the same id that is cached in the current linker instance.
        unsafe {
            self.inner
                .define_module_instance_unchecked(store.inner(), module_name, module)
        }?;
        // 4. rewrap
        // result is the unit type.
        // 5. return
        Ok(())
    }

    /// This is a safe variant of [`Linker::get_unchecked`](crate::linker::Linker::get_unchecked).
    ///
    /// # Interaction with unchecked API
    ///
    /// This method is able to find externs defined through the unchecked
    /// `define` methods.  However, for this to work, at least one of the
    /// following methods must have been called successfully:
    /// [`Linker::define`], [`Linker::define_module_instance`],
    /// [`Linker::module_instantiate`]. Otherwise, this method may spuriously
    /// return an error.
    ///
    /// Therefore, it is advised against to mix the unchecked and checked API
    /// for a single [`Linker`] instance.
    ///
    /// # Errors
    ///
    /// - [`RuntimeError::LinkerNotYetAssociatedWithStoreId`]
    /// - [`RuntimeError::UnableToResolveExternLookup`]
    pub fn get(&self, module_name: String, name: String) -> Result<StoredExternVal, RuntimeError> {
        // 1. get or insert the `StoreId`
        // TODO docs are not consistent
        let Some(linker_store_id) = self.store_id else {
            // At this point we have no way to set the current store id, because
            // the parameters are all non-stored types.

            // We also know that nothing was defined in this linker context through
            // the checked methods yet, because `self.store_id` has not been set
            // yet. Therefore, a get would always return `None`.

            // However, when an unchecked `define` method was used before, we
            // also have to return `None` here, because even if the lookup for
            // `module_name` and `name` returns something, we cannot attach a
            // store id to it.

            return Err(RuntimeError::LinkerNotYetAssociatedWithStoreId);
        };
        // 2. try unwrap
        // no stored parameters
        // 3. call
        let extern_val = self
            .inner
            .get(module_name, name)
            .ok_or(RuntimeError::UnableToResolveExternLookup)?;
        // 4. rewrap
        // SAFETY: The `ExternVal` just came from the current `Linker`. Because
        // a `Linker` can always be used with only one unique `Store`, this
        // `ExternVal` must be from the current Linker's store.
        let stored_extern_val = unsafe { StoredExternVal::from_bare(extern_val, linker_store_id) };
        // 5. return
        Ok(stored_extern_val)
    }

    /// This is a variant of [`Linker::instantiate_pre`](crate::linker::Linker::instantiate_pre).
    ///
    /// # Interaction with unchecked API
    ///
    /// See [`Linker::get`]
    ///
    /// # Errors
    ///
    /// - [`RuntimeError::LinkerNotYetAssociatedWithStoreId`]
    /// - [`RuntimeError::UnableToResolveExternLookup`]
    pub fn instantiate_pre(
        &self,
        validation_info: &ValidationInfo,
    ) -> Result<Vec<StoredExternVal>, RuntimeError> {
        // Special case: If the module has no imports, we don't perform any
        // linking. We need this special case, so that a `Linker`, that has not
        // yet been associated with some `Store`, can still be used to
        // pre-instantiate modules.
        if validation_info.imports().len() == 0 {
            return Ok(Vec::new());
        }
        // 1. get or insert `StoreId`
        let Some(linker_store_id) = self.store_id else {
            // We are not able to perform safe linking (see this method's and
            // `Linker::get`'s documentations).
            return Err(RuntimeError::LinkerNotYetAssociatedWithStoreId);
        };
        // 2. try unwrap
        // no stored parameters
        // 3. call
        let extern_vals = self.inner.instantiate_pre(validation_info)?;
        // 4. rewrap
        // SAFETY: All `ExternVal`s just came from the current `Linker`. Because
        // a Linker can always be used with only one unique `Store`, all
        // `ExternVal`s must be from the current Linker's store.
        let stored_extern_vals = unsafe { Vec::from_bare(extern_vals, linker_store_id) };
        // 5. retur
        Ok(stored_extern_vals)
    }

    /// This is a safe variant of [`Linker::module_instantiate_unchecked`](crate::linker::Linker::module_instantiate_unchecked).
    pub fn module_instantiate<'b, T: Config>(
        &mut self,
        store: &mut Store<'b, T>,
        validation_info: &ValidationInfo<'b>,
        maybe_fuel: Option<u64>,
    ) -> Result<StoredInstantiationOutcome, RuntimeError> {
        // 1. get or insert `StoreId`
        let linker_store_id = *self.store_id.get_or_insert(store.id);
        if linker_store_id != store.id {
            panic!("Store id mismatch");
        }
        // 2. try unwrap
        // no stored parameters
        // 3. call
        // SAFETY: It was just checked that the `ExternVal` came from the store
        // with the same id that is cached in the current linker instance.
        let instantiation_outcome = unsafe {
            self.inner
                .module_instantiate_unchecked(&mut store.inner, validation_info, maybe_fuel)
        }?;
        // 4. rewrap
        // SAFETY: The `InstantiationOutcome` just came from the current
        // `Linker`. Because a linker can always be used with only one unique
        // `Store`, the `InstantiationOutcome` must be from the current Linker's
        // store.
        let stored_instantiation_outcome = unsafe {
            StoredInstantiationOutcome::from_bare(instantiation_outcome, linker_store_id)
        };
        // 5. return
        Ok(stored_instantiation_outcome)
    }
}
