use alloc::{string::String, vec::Vec};
use wasm::{
    addrs::ModuleAddr, config::Config, validation_config::ValidationConfig, RuntimeError,
    ValidationInfo,
};

use crate::{
    store::Store,
    stored_types::{Stored, StoredExternVal, StoredInstantiationOutcome},
    AbstractStored, StoreId,
};

#[derive(Default)]
pub struct Linker {
    inner: linker::Linker,

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

    /// This is a safe variant of [`Linker::define`](linker::Linker::define).
    pub fn define(
        &mut self,
        module_name: String,
        name: String,
        extern_val: StoredExternVal,
    ) -> Result<(), RuntimeError> {
        // 1. get or insert the `StoreId`
        let extern_val_store_id = extern_val.id();
        let linker_store_id = *self.store_id.get_or_insert(extern_val_store_id);
        if linker_store_id != extern_val_store_id {
            panic!("Store id mismatch");
        }
        // 2. try unwrap
        let extern_val = extern_val.try_unwrap_into_bare(linker_store_id);
        // 3. call
        // SAFETY: It was just checked that the `ExternVal` came from the store
        // with the same id that is cached in the current linker instance.
        unsafe { self.inner.define(module_name, name, extern_val) }?;
        // 4. rewrap
        // result is the unit type.
        // 5. return
        Ok(())
    }

    /// This is a safe variant of
    /// [`Linker::define_module_instance`](linker::Linker::define_module_instance).
    pub fn define_module_instance<T: Config>(
        &mut self,
        store: &Store<T>,
        module_name: String,
        module: Stored<ModuleAddr>,
    ) -> Result<(), RuntimeError> {
        // 1. get or insert the `StoreId`
        let module_store_id = module.id();
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
                .define_module_instance(store.inner(), module_name, module)
        }?;
        // 4. rewrap
        // result is the unit type.
        // 5. return
        Ok(())
    }

    /// This is a safe variant of [`Linker::get`](linker::Linker::get).
    pub fn get(&self, module_name: String, name: String) -> Option<StoredExternVal> {
        // 1. get or insert the `StoreId`
        // Note: We can only get the id. If it has not been set yet, no
        // definitions could have been made.
        let linker_store_id = self.store_id?;
        // 2. try unwrap
        // no stored parameters
        // 3. call
        let extern_val = self.inner.get(module_name, name)?;
        // 4. rewrap
        // SAFETY: The `ExternVal` just came from the current `Linker`. Because
        // a `Linker` can always be used with only one unique `Store`, this
        // `ExternVal` must be from the current Linker's store.
        let stored_extern_val = unsafe { StoredExternVal::from_bare(extern_val, linker_store_id) };
        // 5. return
        Some(stored_extern_val)
    }

    /// This is a variant of
    /// [`Linker::instantiate_pre`](linker::Linker::instantiate_pre).
    pub fn instantiate_pre<T: ValidationConfig>(
        &self,
        validation_info: &ValidationInfo<T>,
    ) -> Option<Vec<StoredExternVal>> {
        // Special case: If the module has no imports, we don't perform any
        // linking. We need this special case, so that a `Linker`, that has not
        // yet been associated with some `Store`, can still be used to
        // pre-instantiate modules.
        if validation_info.imports().len() == 0 {
            return Some(Vec::new());
        }
        // 1. get or insert `StoreId`
        // Note: We can only get the id. If it has not been set yet, no
        // definitions could have been made.
        let linker_store_id = self.store_id?;
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
        Some(stored_extern_vals)
    }

    /// This is a safe variant of
    /// [`Linker::module_instantiate`](linker::Linker::module_instantiate).
    pub fn module_instantiate<'b, T: Config, T2: ValidationConfig>(
        &mut self,
        store: &mut Store<'b, T>,
        validation_info: &ValidationInfo<'b, T2>,
        maybe_fuel: Option<u64>,
    ) -> Option<Result<StoredInstantiationOutcome, RuntimeError>> {
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
        let instantiation_outcome = match unsafe {
            self.inner
                .module_instantiate(&mut store.inner, validation_info, maybe_fuel)
        } {
            Some(Ok(instantiation_outcome)) => instantiation_outcome,
            Some(Err(err)) => return Some(Err(err)),
            None => return None,
        };
        // 4. rewrap
        // SAFETY: The `InstantiationOutcome` just came from the current
        // `Linker`. Because a linker can always be used with only one unique
        // `Store`, the `InstantiationOutcome` must be from the current Linker's
        // store.
        let stored_instantiation_outcome = unsafe {
            StoredInstantiationOutcome::from_bare(instantiation_outcome, linker_store_id)
        };
        // 5. return
        Some(Ok(stored_instantiation_outcome))
    }
}
