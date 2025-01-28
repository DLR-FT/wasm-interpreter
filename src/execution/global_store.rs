use alloc::vec;
use alloc::vec::Vec;

use crate::core::reader::types::global::Global;
use crate::core::reader::types::import::ImportRefData;
use crate::core::reader::types::TableType;
use crate::execution::value::Ref;
use crate::execution::value::Value;

#[derive(Default)]
pub struct GlobalStore {
    globals: Vec<GlobalInst>,
    tables: Vec<TableInst>,
}

pub struct GlobalInst {
    pub owner_data: ImportRefData,
    pub global: Global,
    /// Must be of the same type as specified in `ty`
    pub value: Value,
}

#[derive(Debug)]
pub struct TableInst {
    pub owner_data: ImportRefData,
    pub ty: TableType,
    pub elem: Vec<Ref>,
}

impl TableInst {
    pub fn len(&self) -> usize {
        self.elem.len()
    }

    pub fn is_empty(&self) -> bool {
        self.elem.is_empty()
    }

    pub fn new(ty: TableType) -> Self {
        Self {
            ty,
            elem: vec![Ref::default_from_ref_type(ty.et); ty.lim.min as usize],
            owner_data: Default::default(),
        }
    }

    pub fn set_owner_data(&mut self, owner_data: ImportRefData) {
        self.owner_data = owner_data;
    }
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

    pub fn add_table(&mut self, table: TableInst) -> usize {
        self.tables.push(table);
        self.tables.len() - 1
    }

    pub fn get_mut_table(&mut self, table_idx: usize) -> &mut TableInst {
        &mut self.tables[table_idx]
    }

    pub fn get_table(&self, table_idx: usize) -> &TableInst {
        &self.tables[table_idx]
    }

    pub fn get_immut_and_mut_table_pair(
        &mut self,
        // src
        first: usize,
        // dst
        second: usize,
    ) -> (&TableInst, &mut TableInst) {
        use core::cmp::Ordering::*;
        match second.cmp(&first) {
            Greater => {
                let (left, right) = self.tables.split_at_mut(second);
                (&left[first], &mut right[0])
            }
            Less => {
                let (left, right) = self.tables.split_at_mut(first);
                (&right[0], &mut left[second])
            }
            Equal => unreachable!(),
        }
    }

    pub fn find_table_idx(&self, import_ref_data: &ImportRefData) -> usize {
        for (idx, table) in self.tables.iter().enumerate() {
            if *import_ref_data == table.owner_data {
                return idx;
            }
        }
        return usize::MAX;
    }
}
