//! Methods to read WASM Types from a [WasmReader] object.
//!
//! See: <https://webassembly.github.io/spec/core/binary/types.html>

use crate::core::structure::types::{ExternType, Limits};

pub mod data;
pub mod element;
pub mod export;
pub mod global;
pub mod import;
pub mod memarg;
pub mod values;

//https://webassembly.github.io/spec/core/valid/types.html#import-subtyping
pub trait ImportSubTypeRelation {
    // corresponds to "matches" (<=) in the spec
    fn is_subtype_of(&self, other: &Self) -> bool;
}

impl ImportSubTypeRelation for Limits {
    //https://webassembly.github.io/spec/core/valid/types.html#match-limits
    fn is_subtype_of(&self, other: &Self) -> bool {
        (self.min >= other.min)
            && (match other.max {
                None => true,
                Some(other_max) => match self.max {
                    None => false,
                    Some(self_max) => self_max <= other_max,
                },
            })
    }
}

impl ImportSubTypeRelation for ExternType {
    // https://webassembly.github.io/spec/core/valid/types.html#match-limits
    fn is_subtype_of(&self, other: &Self) -> bool {
        match self {
            ExternType::Table(self_table_type) => match other {
                ExternType::Table(other_table_type) => {
                    self_table_type.lim.is_subtype_of(&other_table_type.lim)
                        && self_table_type.et == other_table_type.et
                }
                _ => false,
            },
            ExternType::Mem(self_mem_type) => match other {
                ExternType::Mem(other_mem_type) => {
                    self_mem_type.limits.is_subtype_of(&other_mem_type.limits)
                }
                _ => false,
            },
            _ => self == other,
        }
    }
}
