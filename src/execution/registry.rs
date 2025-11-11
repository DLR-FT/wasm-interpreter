use crate::addrs::ModuleAddr;
use crate::instances::ModuleInst;
use crate::RuntimeError;

use alloc::borrow::Cow;
use alloc::collections::BTreeMap;

use crate::ExternVal;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
struct ImportKey {
    module_name: Cow<'static, str>,
    name: Cow<'static, str>,
}
#[derive(Default, Debug)]
pub struct Registry {
    extern_vals: BTreeMap<ImportKey, ExternVal>,
    modules: BTreeMap<Cow<'static, str>, ModuleAddr>,
}

impl Registry {
    pub fn register(
        &mut self,
        module_name: Cow<'static, str>,
        name: Cow<'static, str>,
        extern_val: ExternVal,
    ) -> Result<(), RuntimeError> {
        if self
            .extern_vals
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
        self.extern_vals
            .get(&ImportKey { module_name, name })
            .ok_or(RuntimeError::UnknownImport)
    }

    pub fn lookup_module(&self, module_name: Cow<'static, str>) -> Option<ModuleAddr> {
        self.modules.get(&module_name).copied()
    }

    pub fn register_module(
        &mut self,
        module_name: Cow<'static, str>,
        module_inst: &ModuleInst,
        module_addr: ModuleAddr,
    ) -> Result<(), RuntimeError> {
        self.modules.insert(module_name.clone(), module_addr);

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
