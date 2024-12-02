use core::fmt::Debug;

use alloc::vec::Vec;

use crate::{
    core::reader::span::Span, read_constant_expression::read_constant_instructions, Error, Result,
};

use super::RefType;

#[derive(Clone)]
pub struct ElemType {
    pub init: ElemItems,
    pub mode: ElemMode,
}

impl Debug for ElemType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "ElemType {{\n\tinit: {:?},\n\tmode: {:?},\n\t#ty: {}\n}}",
            self.init,
            self.mode,
            self.init.ty()
        )
    }
}

impl ElemType {
    pub fn ty(&self) -> RefType {
        self.init.ty()
    }
    pub fn to_ref_type(&self) -> RefType {
        match self.init {
            ElemItems::Exprs(rref, _) => rref,
            ElemItems::RefFuncs(_) => RefType::FuncRef,
        }
    }

    /// Here we can't implement WasmReadable because we also want a mutable
    /// reference to a BTreeSet<u32> (`referenced_functions`)
    ///
    /// This comes in handy later on when we are validating the actual code of
    /// the functions so that we can make sure we are not referencing invalid functions
    pub fn read_from_wasm(
        wasm: &mut crate::core::reader::WasmReader,
        functions: &[usize],
        referenced_functions: &mut alloc::collections::btree_set::BTreeSet<u32>,
        tables_length: usize,
    ) -> Result<Vec<Self>> {
        use crate::core::reader::types::element::*;
        use crate::RefType;
        // https://webassembly.github.io/spec/core/binary/modules.html#element-section

        wasm.read_vec(|wasm| {
            let ty = wasm.read_var_u32().unwrap();
            // https://webassembly.github.io/spec/core/syntax/modules.html#element-segments
            // https://webassembly.github.io/spec/core/binary/modules.html#element-section
            // We can treat the ttype as a 3bit integer
            // If it's not 3 bits I am not sure what to do
            // bit 0 => diff between passive|declartive and active segment
            // bit 1 => presence of an explicit table index for an active segment
            // bit 2 => use of element type and element expressions instead of element kind and element indices
            assert!(ty <= 0b111, "Element section is not encoded correctly. The type of this element is over 7 (0b111)");
            // decide if we should
            let elem_mode = if ty & 0b001 == 0b001 {
                if ty & 0b010 == 0b010 {
                    ElemMode::Declarative
                } else {
                    ElemMode::Passive
                }
            } else {
                let table_idx = if ty & 0b010 == 0b010 {
                    wasm.read_var_u32()?
                } else {
                    0
                };
                if tables_length <= table_idx as usize {
                    return Err(Error::UnknownTable);
                }
                let expr = read_constant_instructions(wasm, None, None, Some(functions))?;

                ElemMode::Active(ActiveElem {
                    table: table_idx,
                    offset: expr,
                })
            };
            let use_of_el_ty_and_el_exprs = ty & 0b100 == 0b100;

            let reftype_or_elemkind: Option<RefType> = match if ty & 0b011 != 0 {
                if use_of_el_ty_and_el_exprs {
                    Some(wasm.read_u8()?)
                } else {
                    let read = wasm.read_u8()?;
                    match read {
                        0x00 => None,
                        _ => todo!("Only FuncRefs are allowed"),
                    }
                }
            } else {
                None
            } {
                None => None,
                Some(ty) => Some(RefType::from_byte(ty)?),
            };

            match reftype_or_elemkind {
                Some(rty) => trace!("REFTYPE: {}", rty),
                None => {
                    trace!("REFTYPE NONE!")
                }
            };

            let items: ElemItems = if use_of_el_ty_and_el_exprs {
                ElemItems::Exprs(
                    reftype_or_elemkind.unwrap_or(RefType::FuncRef),
                    wasm.read_vec(|w| read_constant_instructions(w, None, None, Some(functions)))?,
                )
            } else {
                assert!(reftype_or_elemkind.is_none());
                ElemItems::RefFuncs(wasm.read_vec(|w| {
                    let offset = w.read_var_u32()?;
                    referenced_functions.insert(offset);
                    Ok(offset)
                })?)
            };

            let el = ElemType {
                init: items,
                mode: elem_mode,
            };

            Ok(el)
        })
    }
}

#[derive(Debug, Clone)]
pub enum ElemItems {
    RefFuncs(Vec<u32>),
    Exprs(RefType, Vec<Span>),
}

impl ElemItems {
    pub fn ty(&self) -> RefType {
        match self {
            Self::RefFuncs(_) => RefType::FuncRef,
            Self::Exprs(rty, _) => *rty,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ElemMode {
    Passive,
    Active(ActiveElem),
    Declarative,
}

#[derive(Debug, Clone)]
pub struct ActiveElem {
    pub table: u32,
    pub offset: Span,
}
