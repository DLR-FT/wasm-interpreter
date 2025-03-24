use super::global::GlobalType;
use super::RefType;
use crate::core::reader::span::Span;
use crate::core::reader::WasmReader;
use crate::read_constant_expression::read_constant_expression;
use crate::validation_stack::ValidationStack;
use crate::{Error, Result};

use alloc::collections::btree_set::BTreeSet;
use alloc::vec::Vec;
use core::fmt::Debug;

#[derive(Clone)]
pub struct ElemType {
    pub init: ElemItems,
    pub mode: ElemMode,
}

impl Debug for ElemType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "ElemType {{\n\tinit: {:?},\n\tmode: {:?},\n\t#ty: {:?}\n}}",
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

    // TODO: @nerodesu017 maybe split up the validation from the parsing?
    /// Here we can't implement WasmReadable because we also want a mutable
    /// reference to a [`BTreeSet<u32>`] (`referenced_functions`)
    ///
    /// This comes in handy later on when we are validating the actual code of
    /// the functions so that we can make sure we are not referencing invalid
    /// functions
    pub fn read_from_wasm(
        wasm: &mut WasmReader,
        functions: &[usize],
        referenced_functions: &mut BTreeSet<u32>,
        tables_length: usize,
        all_globals_types: &[GlobalType],
    ) -> Result<Vec<Self>> {
        wasm.read_vec(|wasm| {
            let prop = wasm.read_var_u32()?;

            // TODO: @nerodesu017 revisit this comment
            // https://webassembly.github.io/spec/core/syntax/modules.html#element-segments
            // https://webassembly.github.io/spec/core/binary/modules.html#element-section
            // We can treat the ty as a 3bit integer
            // If it's not 3 bits I am not sure what to do
            // bit 0 => diff between passive|declartive and active segment
            // bit 1 => presence of an explicit table index for an active segment
            // bit 2 => use of element type and element expressions instead of element kind and element indices
            // decide if we should

            // TODO: @nerodesu017 error, this is a parse error, not validation FYI
            // NOTE: This assert breaks my rustfmt :(
            // assert!(prop <= 0b111, "Element section is not encoded correctly. The type of this element is over 7 (0b111)");

            let elem_mode = if prop & 0b011 == 0b011 {
                ElemMode::Declarative
            } else if prop & 0b001 == 0b001 {
                ElemMode::Passive
            } else {
                let table_idx = if prop & 0b010 == 0b010 {
                    wasm.read_var_u32()?
                } else {
                    0
                };

                if tables_length <= table_idx as usize {
                    return Err(Error::UnknownTable);
                }

                let mut valid_stack = ValidationStack::new();
                // let wasm_pc = wasm.pc;
                let init_expr = read_constant_expression(
                    wasm,
                    &mut valid_stack,
                    all_globals_types,
                    Some(functions),
                )?;

                // at validation time we actually have to check for the function to be known
                //  that means a new stack
                //  nvm we will do that directly at runtime
                //  we could do a part now, for example, if we don't have imported globals to take care of, just so we are even more
                //  reliable (more checks at validation/compile time)

                // now, why do I mention this? most of the testing is done online on the https://webassembly.github.io/wabt/demo/wat2wasm/ website
                //  chromium's wasm engine actually catches (as much as it can) errors at validation (tries to run the expressions)

                // on top of the stack it's supposed to be the
                // it might also be an extern ref w/e
                // let function_idx = valid_stack.peek_const_validation_stack();

                // match function_idx {
                //     None => return Err(Error::UnknownFunction),
                //     Some(stack_entry) => match stack_entry {
                //         crate::validation_stack::ValidationStackEntry::Val(val) => match val {
                //             super::ValType::NumType(num) => match num {
                //                 super::NumType::I32 => {
                //                     wasm.pc = wasm_pc;
                //                     let stack = &mut Stack::new();
                //                     run_const(wasm, stack, ());
                //                     let popped_from_stack: u32 = stack
                //                         .pop_value(super::ValType::NumType(super::NumType::I32))
                //                         .into();
                //                     if functions.len() >= popped_from_stack as usize {
                //                         return Err(Error::UnknownFunction);
                //                     }
                //                 }
                //                 _ => return Err(Error::UnknownFunction),
                //             },
                //             _ => return Err(Error::UnknownFunction),
                //         },
                //         _ => return Err(Error::UnknownFunction),
                //     },
                // };

                valid_stack.assert_pop_val_type(super::ValType::NumType(super::NumType::I32))?;

                ElemMode::Active(ActiveElem {
                    table_idx,
                    init_expr,
                })
            };

            let third_bit_set = prop & 0b100 == 0b100;

            let type_kind = if prop & 0b011 != 0 {
                if third_bit_set {
                    Some(wasm.read_u8()?)
                } else {
                    match wasm.read_u8()? {
                        0x00 => None,
                        _ => return Err(Error::OnlyFuncRefIsAllowed),
                    }
                }
            } else {
                None
            };

            let reftype_or_elemkind: Option<RefType> = match type_kind {
                Some(ty) => Some(RefType::from_byte(ty)?),
                None => None,
            };

            let items: ElemItems = if third_bit_set {
                ElemItems::Exprs(
                    reftype_or_elemkind.unwrap_or(RefType::FuncRef),
                    wasm.read_vec(|w| {
                        let mut valid_stack = ValidationStack::new();
                        let span = read_constant_expression(
                            w,
                            &mut valid_stack,
                            all_globals_types,
                            Some(functions),
                        );

                        use crate::validation_stack::ValidationStackEntry::*;

                        if let Some(val) = valid_stack.peek_const_validation_stack() {
                            if let Val(val) = val {
                                match val {
                                    crate::ValType::RefType(_) => {}
                                    crate::ValType::NumType(crate::NumType::I32) => {}
                                    crate::ValType::NumType(_) => {
                                        return Err(Error::InvalidValidationStackValType(Some(val)))
                                    }
                                    _ => {
                                        return Err(Error::InvalidValidationStackValType(Some(val)))
                                    }
                                }
                            } else {
                                return Err(Error::InvalidValidationStackType(val));
                            }
                        } else {
                            return Err(Error::InvalidValidationStackValType(None));
                        }

                        span
                    })?,
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
    pub table_idx: u32,
    pub init_expr: Span,
}
