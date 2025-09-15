use crate::{ModuleInst, RuntimeError};

use alloc::borrow::Cow;
use alloc::collections::BTreeMap;

use crate::ExternVal;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
struct ImportKey {
    module_name: Cow<'static, str>,
    name: Cow<'static, str>,
}
#[derive(Default, Debug)]
pub struct Registry(BTreeMap<ImportKey, ExternVal>);

impl Registry {
    pub fn register(
        &mut self,
        module_name: Cow<'static, str>,
        name: Cow<'static, str>,
        extern_val: ExternVal,
    ) -> Result<(), RuntimeError> {
        if self
            .0
            .insert(ImportKey { module_name, name }, extern_val)
            .is_some()
        {
            return Err(RuntimeError::InvalidImportType);
        }

        Ok(())
    }

    pub fn lookup(
        &self,
        module_name: Cow<'static, str>,
        name: Cow<'static, str>,
    ) -> Result<&ExternVal, RuntimeError> {
        // Note: We cannot do a `&str` lookup on a [`String`] map key.
        // Thus we have to use `Cow<'static, str>` as a key
        // (at least this prevents allocations with static names).
        self.0
            .get(&ImportKey { module_name, name })
            .ok_or(RuntimeError::UnknownImport)
    }

    pub fn register_module(
        &mut self,
        module_name: Cow<'static, str>,
        module_inst: &ModuleInst,
    ) -> Result<(), RuntimeError> {
        for (entity_name, extern_val) in &module_inst.exports {
            // FIXME this clones module_name. Maybe prevent by using `Cow<'static, Arc<str>>`.
            self.register(
                module_name.clone(),
                Cow::Owned(entity_name.clone()),
                *extern_val,
            )?;
        }
        Ok(())
    }
}
