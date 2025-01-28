use alloc::vec::Vec;

use crate::core::reader::types::global::Global;
use crate::core::reader::types::import::ImportRefData;
use crate::execution::value::Value;

#[derive(Default)]
pub struct GlobalStore {
    globals: Vec<GlobalInst>,
}

pub struct GlobalInst {
    pub owner_data: ImportRefData,
    pub global: Global,
    /// Must be of the same type as specified in `ty`
    pub value: Value,
}

impl GlobalStore {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_global(&mut self, global: GlobalInst) -> usize {
        self.globals.push(global);
        self.globals.len() - 1
    }

    pub fn get_mut_global(&mut self, global_idx: usize) -> &mut GlobalInst {
        &mut self.globals[global_idx]
    }

    pub fn get_global(&self, global_idx: usize) -> &GlobalInst {
        &self.globals[global_idx]
    }

    pub fn find_global_idx(&self, import_ref_data: &ImportRefData) -> usize {
        for (idx, global) in self.globals.iter().enumerate() {
            if *import_ref_data == global.owner_data {
                return idx;
            }
        }
        return usize::MAX;
    }
}
