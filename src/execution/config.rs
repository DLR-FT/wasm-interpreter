use crate::hooks::{EmptyHookSet, HookSet};

pub trait Config {
    type HookSet: HookSet;

    fn hook_set(&self) -> &Self::HookSet;
    fn hook_set_mut(&mut self) -> &mut Self::HookSet;
}

#[derive(Default, Debug)]
pub struct DefaultConfig {
    hooks: EmptyHookSet,
}

impl Config for DefaultConfig {
    type HookSet = EmptyHookSet;

    fn hook_set(&self) -> &Self::HookSet {
        &self.hooks
    }

    fn hook_set_mut(&mut self) -> &mut Self::HookSet {
        &mut self.hooks
    }
}
