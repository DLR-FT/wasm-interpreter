use crate::{core::structure::types::ExternTypeRef, ExternType, Limits};

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
            && self.shared == other.shared
    }
}

impl ImportSubTypeRelation for ExternType {
    // https://webassembly.github.io/spec/core/valid/types.html#match-limits
    fn is_subtype_of(&self, other: &Self) -> bool {
        self.as_ref().is_subtype_of(&other.as_ref())
    }
}

impl ImportSubTypeRelation for ExternTypeRef<'_> {
    // https://webassembly.github.io/spec/core/valid/types.html#match-limits
    fn is_subtype_of(&self, other: &Self) -> bool {
        match self {
            ExternTypeRef::Table(self_table_type) => match other {
                ExternTypeRef::Table(other_table_type) => {
                    self_table_type.lim.is_subtype_of(&other_table_type.lim)
                        && self_table_type.et == other_table_type.et
                }
                _ => false,
            },
            ExternTypeRef::Mem(self_mem_type) => match other {
                ExternTypeRef::Mem(other_mem_type) => {
                    self_mem_type.limits.is_subtype_of(&other_mem_type.limits)
                }
                _ => false,
            },
            _ => self == other,
        }
    }
}
