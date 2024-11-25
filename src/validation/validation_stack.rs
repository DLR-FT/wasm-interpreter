//! This module contains the [`ValidationStack`] data structure
//!
//! The [`ValidationStack`] is a unified stack, in the sense that it unifies both
//! [`ValidationStackEntry::Val`] and [`ValidationStackEntry::Label`]. It therefore mixes type
//! information with structured control flow information.
#![allow(unused)] // TODO remove this once sidetable implementation lands
use super::Result;
use alloc::vec;
use alloc::vec::Vec;

use crate::{
    core::reader::types::{FuncType, ResultType},
    Error, ValType,
};

#[derive(Debug, PartialEq, Eq)]
pub(super) struct ValidationStack {
    stack: Vec<ValType>,
    // TODO hide implementation
    pub ctrl_stack: Vec<CtrlStackEntry>,
}

impl ValidationStack {
    /// Initialize a new ValidationStack
    pub(super) fn new() -> Self {
        Self {
            stack: Vec::new(),
            ctrl_stack: vec![
                // TODO populate bottom label with the outer func type and arity
                CtrlStackEntry {
                    label_info: LabelInfo::Func,
                    block_ty: FuncType {
                        params: ResultType {
                            valtypes: Vec::new(),
                        },
                        returns: ResultType {
                            valtypes: Vec::new(),
                        },
                    },
                    height: 0,
                    unreachable: false,
                },
            ],
        }
    }

    pub(super) fn len(&self) -> usize {
        self.stack.len()
    }

    pub(super) fn push_valtype(&mut self, valtype: ValType) {
        self.stack.push(valtype);
    }

    /// Similar to [`ValidationStack::pop`], because it pops a value from the stack,
    /// but more public and doesn't actually return the popped value.
    pub(super) fn drop_val(&mut self) -> Result<()> {
        self.pop_valtype().map_err(|e| Error::ExpectedAnOperand)?;
        Ok(())
    }

    pub(super) fn make_unspecified(&mut self) {
        // TODO unwrapping might not be the best option
        let last_ctrl_stack_entry = self.ctrl_stack.last_mut().unwrap();
        last_ctrl_stack_entry.unreachable = true;
        self.stack.truncate(last_ctrl_stack_entry.height)
    }

    /// Pop a [`ValidationStackEntry`] from the [`ValidationStack`]
    ///
    /// # Returns
    ///
    /// - Returns `Ok(_)` with the former top-most [`ValidationStackEntry`] inside, if the stack had
    ///   at least one element.
    /// - Returns `Err(_)` if the stack was already empty.
    // any method that might return Unknown should be private
    fn pop_valtype(&mut self) -> Result<ValidationStackEntry> {
        // TODO unwrapping might not be the best option
        // TODO ugly
        // TODO return type should be Result<()> maybe?
        let last_ctrl_stack_entry = self.ctrl_stack.last().unwrap();
        assert!(self.stack.len() >= last_ctrl_stack_entry.height);
        if last_ctrl_stack_entry.height == self.stack.len() {
            if last_ctrl_stack_entry.unreachable {
                Ok(ValidationStackEntry::UnspecifiedValTypes)
            } else {
                Err(Error::EndInvalidValueStack)
            }
        } else {
            //empty stack is covered with above check
            Ok(ValidationStackEntry::Val(self.stack.pop().unwrap()))
        }
    }

    /// Assert the top-most [`ValidationStackEntry`] is a specific [`ValType`], after popping it from the [`ValidationStack`]
    ///
    /// # Returns
    ///
    /// - Returns `Ok(())` if the top-most [`ValidationStackEntry`] is a [`ValType`] identical to
    ///   `expected_ty`.
    /// - Returns `Err(_)` otherwise.
    ///
    pub(super) fn assert_pop_val_type(&mut self, expected_ty: ValType) -> Result<()> {
        match self.pop_valtype()? {
            ValidationStackEntry::Val(ty) => (ty == expected_ty)
                .then_some(())
                .ok_or(Error::InvalidValidationStackValType(Some(ty))),
            ValidationStackEntry::UnspecifiedValTypes => Ok(()),
        }
    }

    /// Asserts that the values on top of the stack match those of a value iterator
    ///
    /// The last element of `expected_val_types` is compared to the top-most
    /// [`ValidationStackEntry`], the second last `expected_val_types` element to the second top-most
    /// [`ValidationStackEntry`] etc.
    ///
    /// Any occurence of the [`ValidationStackEntry::Label`] variant in the stack tail will cause an
    /// error. This method does not mutate the [`ValidationStack::stack`] in any way.
    ///
    /// # Returns
    ///
    /// - `Ok(_)`, the tail of the stack matches the `expected_val_types`
    /// - `Err(_)` otherwise
    pub(super) fn assert_val_types_on_top(&self, expected_val_types: &[ValType]) -> Result<()> {
        // TODO unwrapping might not be the best option
        let last_ctrl_stack_entry = self.ctrl_stack.last().unwrap();

        for (i, expected_ty) in expected_val_types.iter().rev().enumerate() {
            if self.stack.len() - last_ctrl_stack_entry.height <= i {
                if last_ctrl_stack_entry.unreachable {
                    return Ok(());
                } else {
                    return Err(Error::EndInvalidValueStack);
                }
            }

            // this access won't blow up because of the above check
            if self.stack[self.stack.len() - i - 1] != *expected_ty {
                return Err(Error::InvalidValidationStackValType(Some(
                    self.stack[self.stack.len() - i - 1],
                )));
            }
        }
        Ok(())
    }

    /// Asserts that the valtypes on the stack match the expected valtypes.
    ///
    /// This starts by comparing the top-most valtype with the last element from `expected_val_types` and then continues downwards on the stack.
    /// If a label is reached and not all `expected_val_types` have been checked, the assertion fails.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if all expected valtypes were found
    /// - `Err(_)` otherwise
    pub(super) fn assert_val_types(&self, expected_val_types: &[ValType]) -> Result<()> {
        // TODO unwrapping might not be the best option
        let last_ctrl_stack_entry = self.ctrl_stack.last().unwrap();

        if self.stack.len() - last_ctrl_stack_entry.height != expected_val_types.len() {
            return Err(Error::EndInvalidValueStack);
        }

        self.assert_val_types_on_top(expected_val_types)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ValidationStackEntry {
    /// A value
    Val(ValType),
    /// Special variant to encode that any possible number of [`ValType`]s could be here
    ///
    /// Caused by `return` and `unreachable`, as both can push an arbitrary number of values to the stack.
    ///
    /// When this variant is pushed onto the stack, all valtypes until the next lower label are deleted.
    /// They are not needed anymore because this variant can expand to all of them.
    UnspecifiedValTypes,
}
// TODO hide implementation
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CtrlStackEntry {
    pub label_info: LabelInfo,
    pub block_ty: FuncType,
    pub height: usize,
    pub unreachable: bool,
}

// TODO replace LabelInfo with this
// TODO hide implementation
// TODO implementation coupled to Sidetable
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LabelInfo {
    Block {
        stps_to_backpatch: Vec<usize>,
    },
    Loop {
        ip: usize,
        stp: usize,
    },
    If {
        stps_to_backpatch: Vec<usize>,
        stp: usize,
    },
    Func,
}

impl LabelInfo {
    fn label_kind(&self) -> LabelKind {
        match self {
            LabelInfo::Block { .. } => LabelKind::Block,
            LabelInfo::Loop { .. } => LabelKind::Loop,
            LabelInfo::If { .. } => LabelKind::If,
            LabelInfo::Func => LabelKind::Func,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LabelKind {
    Block,
    Loop,
    If,
    Func,
}

#[cfg(test)]
mod tests {
    use crate::{NumType, RefType, ValType};

    use super::{CtrlStackEntry, FuncType, LabelInfo, LabelKind, ResultType, ValidationStack, Vec};

    // TODO remove this later
    fn push_dummy_func_label(validation_stack: &mut ValidationStack) {
        validation_stack.ctrl_stack.push(CtrlStackEntry {
            label_info: LabelInfo::Func,
            block_ty: FuncType {
                params: ResultType {
                    valtypes: Vec::new(),
                },
                returns: ResultType {
                    valtypes: Vec::new(),
                },
            },
            height: validation_stack.len(),
            unreachable: false,
        })
    }

    fn push_dummy_block_label(validation_stack: &mut ValidationStack) {
        validation_stack.ctrl_stack.push(CtrlStackEntry {
            label_info: LabelInfo::Block {
                stps_to_backpatch: Vec::new(),
            },
            block_ty: FuncType {
                params: ResultType {
                    valtypes: Vec::new(),
                },
                returns: ResultType {
                    valtypes: Vec::new(),
                },
            },
            height: validation_stack.len(),
            unreachable: false,
        })
    }

    #[test]
    fn push_then_pop() {
        let mut stack = ValidationStack::new();

        stack.push_valtype(ValType::NumType(NumType::F64));
        stack.push_valtype(ValType::NumType(NumType::I32));
        stack.push_valtype(ValType::VecType);
        stack.push_valtype(ValType::RefType(RefType::ExternRef));

        stack
            .assert_pop_val_type(ValType::RefType(RefType::ExternRef))
            .unwrap();
        stack.assert_pop_val_type(ValType::VecType).unwrap();
        stack
            .assert_pop_val_type(ValType::NumType(NumType::I32))
            .unwrap();
        stack
            .assert_pop_val_type(ValType::NumType(NumType::F64))
            .unwrap();
    }

    // TODO rewrite these
    // #[test]
    // fn labels() {
    //     let mut stack = ValidationStack::new();

    //     stack.push_valtype(ValType::NumType(NumType::I64));
    //     push_dummy_func_label(&mut stack);

    //     push_dummy_block_label(&mut stack);

    //     stack.push_valtype(ValType::VecType);

    //     // This removes the `ValType::VecType` and the `LabelKind::Loop` label
    //     let popped_label = stack.pop_label_and_above().unwrap();
    //     assert_eq!(
    //         popped_label,
    //         LabelInfo {
    //             kind: LabelKind::Loop,
    //         }
    //     );

    //     let popped_label = stack.pop_label_and_above().unwrap();
    //     assert_eq!(
    //         popped_label,
    //         LabelInfo {
    //             kind: LabelKind::Block,
    //         }
    //     );

    //     // The first valtype should still be there
    //     stack.assert_pop_val_type(ValType::NumType(NumType::I64));
    // }

    #[test]
    fn assert_valtypes() {
        let mut stack = ValidationStack::new();

        stack.push_valtype(ValType::NumType(NumType::F64));
        stack.push_valtype(ValType::NumType(NumType::I32));
        stack.push_valtype(ValType::NumType(NumType::F32));

        stack
            .assert_val_types(&[
                ValType::NumType(NumType::F64),
                ValType::NumType(NumType::I32),
                ValType::NumType(NumType::F32),
            ])
            .unwrap();

        push_dummy_func_label(&mut stack);

        stack.push_valtype(ValType::NumType(NumType::I32));

        stack
            .assert_val_types(&[ValType::NumType(NumType::I32)])
            .unwrap();
    }

    #[test]
    fn assert_emtpy_valtypes() {
        let mut stack = ValidationStack::new();

        stack.assert_val_types(&[]).unwrap();

        stack.push_valtype(ValType::NumType(NumType::I32));
        push_dummy_func_label(&mut stack);

        // Valtypes separated by a label should also not be detected
        stack.assert_val_types(&[]).unwrap();
    }

    #[test]
    fn assert_valtypes_on_top() {
        let mut stack = ValidationStack::new();

        stack.assert_val_types_on_top(&[]).unwrap();

        stack.push_valtype(ValType::NumType(NumType::I32));
        stack.push_valtype(ValType::NumType(NumType::F32));
        stack.push_valtype(ValType::NumType(NumType::I64));

        // There are always zero valtypes on top of the stack
        stack.assert_val_types_on_top(&[]).unwrap();

        stack
            .assert_val_types_on_top(&[ValType::NumType(NumType::I64)])
            .unwrap();

        stack
            .assert_val_types_on_top(&[
                ValType::NumType(NumType::F32),
                ValType::NumType(NumType::I64),
            ])
            .unwrap();

        stack
            .assert_val_types_on_top(&[
                ValType::NumType(NumType::I32),
                ValType::NumType(NumType::F32),
                ValType::NumType(NumType::I64),
            ])
            .unwrap();
    }

    #[test]
    fn unspecified() {
        let mut stack = ValidationStack::new();
        push_dummy_func_label(&mut stack);

        stack.make_unspecified();

        // Now we can pop as many valtypes from the stack as we want
        stack
            .assert_pop_val_type(ValType::NumType(NumType::I32))
            .unwrap();

        stack
            .assert_pop_val_type(ValType::RefType(RefType::ExternRef))
            .unwrap();

        // Let's remove the unspecified entry and the first label

        // TODO hide implementation
        stack.ctrl_stack.pop();

        // Now there are no values left on the stack
        assert_eq!(stack.assert_val_types(&[]), Ok(()));
    }
}
