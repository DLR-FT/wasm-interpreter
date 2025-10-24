use crate::hooks::{EmptyHookSet, HookSet};

pub trait Config {
    type HookSet: HookSet;
    type UserData;

    fn hook_set(&self) -> &Self::HookSet;
    fn hook_set_mut(&mut self) -> &mut Self::HookSet;

    fn user_data(&self) -> &Self::UserData;
    fn user_data_mut(&mut self) -> &mut Self::UserData;
}

#[derive(Default, Debug)]
pub struct DefaultConfig {
    hooks: EmptyHookSet,
    user_data: (),
}

impl Config for DefaultConfig {
    type HookSet = EmptyHookSet;
    type UserData = ();

    fn hook_set(&self) -> &Self::HookSet {
        &self.hooks
    }

    fn hook_set_mut(&mut self) -> &mut Self::HookSet {
        &mut self.hooks
    }

    fn user_data(&self) -> &Self::UserData {
        &self.user_data
    }

    fn user_data_mut(&mut self) -> &mut Self::UserData {
        &mut self.user_data
    }
}
