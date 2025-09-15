use core::iter;

use alloc::vec;
use alloc::vec::Vec;

use crate::{
    core::reader::types::{FuncType, ResultType},
    Error, NumType, RefType, ValType,
};

#[derive(Debug, PartialEq, Eq)]
pub struct ValidationStack {
    stack: Vec<ValidationStackEntry>,
    // TODO hide implementation
    pub ctrl_stack: Vec<CtrlStackEntry>,
}

impl ValidationStack {
    /// Initialize a new ValidationStack to validate a block of type [] -> []
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            ctrl_stack: vec![CtrlStackEntry {
                label_info: LabelInfo::Untyped,
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
            }],
        }
    }

    /// Initialize a new ValidationStack to validate a block of type `block_ty`
    pub(super) fn new_for_func(block_ty: FuncType) -> Self {
        Self {
            stack: Vec::new(),
            ctrl_stack: vec![CtrlStackEntry {
                label_info: LabelInfo::Func {
                    stps_to_backpatch: Vec::new(),
                },
                block_ty,
                height: 0,
                unreachable: false,
            }],
        }
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }

    pub fn push_valtype(&mut self, valtype: ValType) {
        self.stack.push(ValidationStackEntry::Val(valtype));
    }

    /// Similar to [`ValidationStack::pop_valtype`], because it pops a value from the stack,
    /// but more public and doesn't actually return the popped value.
    pub(super) fn drop_val(&mut self) -> Result<(), Error> {
        self.pop_valtype().map_err(|_| Error::ExpectedAnOperand)?;
        Ok(())
    }

    /// Mark the current control block as unreachable, removing all of the types pushed to the stack since the current control block was entered.
    /// pop operations from the stack will yield `Ok(ValidationStackEntry::Bottom)` if the stack height is the same as the height when this control
    /// block was entered.
    ///
    /// Returns `Ok(())` if called during validation of a control block. `Returns Err(Error::ValidationCtrlStackEmpty)` if no control block context is found
    /// in the control block stack.
    pub(super) fn make_unspecified(&mut self) -> Result<(), Error> {
        let last_ctrl_stack_entry = self
            .ctrl_stack
            .last_mut()
            .ok_or(Error::ValidationCtrlStackEmpty)?;
        last_ctrl_stack_entry.unreachable = true;
        self.stack.truncate(last_ctrl_stack_entry.height);
        Ok(())
    }

    /// Pop a [`ValidationStackEntry`] from the [`ValidationStack`]
    ///
    /// # Returns
    ///
    /// - Returns `Ok(_)` with the former top-most [`ValidationStackEntry`] inside, if the stack had
    ///   at least one element pushed after the current control block is entered. May also return `Ok(ValidationStackEntry::Bottom)`
    ///   if `make_unspecified` is called within the current control block.
    /// - Returns `Err(_)` otherwise.
    fn pop_valtype(&mut self) -> Result<ValidationStackEntry, Error> {
        // TODO unwrapping might not be the best option
        // TODO ugly
        // TODO return type should be Result<()> maybe?
        let last_ctrl_stack_entry = self.ctrl_stack.last().unwrap();
        assert!(self.stack.len() >= last_ctrl_stack_entry.height);
        if last_ctrl_stack_entry.height == self.stack.len() {
            if last_ctrl_stack_entry.unreachable {
                Ok(ValidationStackEntry::Bottom)
            } else {
                Err(Error::EndInvalidValueStack)
            }
        } else {
            //empty stack is covered with above check
            self.stack.pop().ok_or(Error::EndInvalidValueStack)
        }
    }

    /// Attempt popping `Valtype::RefType(expected_ty)` from type stack.
    ///
    /// # Returns
    ///
    /// - Returns `Ok(())` if `Valtype::RefType(expected_ty)` unifies to the item returned by `pop_valtype` operation and `Err(_)` otherwise.
    pub fn assert_pop_ref_type(&mut self, expected_ty: Option<RefType>) -> Result<(), Error> {
        match self.pop_valtype()? {
            ValidationStackEntry::Val(ValType::RefType(ref_type)) => {
                expected_ty.map_or(Ok(()), |ty| {
                    (ty == ref_type)
                        .then_some(())
                        .ok_or(Error::DifferentRefTypes(ref_type, ty))
                })
            }
            ValidationStackEntry::Val(v) => Err(Error::ExpectedARefType(v)),
            ValidationStackEntry::Bottom => Ok(()),
        }
    }

    /// Attempt popping expected_ty from type stack.
    ///
    /// # Returns
    ///
    /// - Returns `Ok(())` if expected_ty unifies to the item returned by `pop_valtype` operation and `Err(_)` otherwise.
    pub fn assert_pop_val_type(&mut self, expected_ty: ValType) -> Result<(), Error> {
        match self.pop_valtype()? {
            ValidationStackEntry::Val(ty) => (ty == expected_ty)
                .then_some(())
                .ok_or(Error::InvalidValidationStackValType(Some(ty))),
            ValidationStackEntry::Bottom => Ok(()),
        }
    }

    // private fns to shut the borrow checker up when calling methods with mutable ref to self with immutable ref to self arguments
    // TODO ugly but I can't come up with anything else better
    fn assert_val_types_on_top_with_custom_stacks(
        stack: &mut Vec<ValidationStackEntry>,
        ctrl_stack: &[CtrlStackEntry],
        expected_val_types: &[ValType],
        unify_to_expected_types: bool,
    ) -> Result<(), Error> {
        let last_ctrl_stack_entry = ctrl_stack.last().ok_or(Error::ValidationCtrlStackEmpty)?;
        let stack_len = stack.len();

        let rev_iterator = expected_val_types.iter().rev().enumerate();
        for (i, expected_ty) in rev_iterator {
            if stack_len - last_ctrl_stack_entry.height <= i {
                if last_ctrl_stack_entry.unreachable {
                    if unify_to_expected_types {
                        // Unify(t2*,expected_val_types) := [t2* expected_val_types]
                        stack.splice(
                            stack_len - i..stack_len - i,
                            expected_val_types[..expected_val_types.len() - i]
                                .iter()
                                .map(|ty| ValidationStackEntry::Val(*ty)),
                        );
                    } else {
                        stack.splice(
                            stack_len - i..stack_len - i,
                            iter::repeat(ValidationStackEntry::Bottom)
                                .take(expected_val_types.len() - i),
                        );
                    }
                    return Ok(());
                } else {
                    return Err(Error::EndInvalidValueStack);
                }
            }

            // the above height check ensures this access is valid
            let actual_ty = &mut stack[stack_len - i - 1];

            match actual_ty {
                ValidationStackEntry::Val(actual_val_ty) => {
                    if *actual_val_ty != *expected_ty {
                        return Err(Error::EndInvalidValueStack);
                    }
                }
                ValidationStackEntry::Bottom => {
                    // Bottom will always unify to the expected ty
                    if unify_to_expected_types {
                        *actual_ty = ValidationStackEntry::Val(*expected_ty);
                    }
                }
            }
        }

        Ok(())
    }

    fn assert_val_types_with_custom_stacks(
        stack: &mut Vec<ValidationStackEntry>,
        ctrl_stack: &[CtrlStackEntry],
        expected_val_types: &[ValType],
        unify_to_expected_types: bool,
    ) -> Result<(), Error> {
        ValidationStack::assert_val_types_on_top_with_custom_stacks(
            stack,
            ctrl_stack,
            expected_val_types,
            unify_to_expected_types,
        )?;
        //if we can assert types in the above there is a last ctrl stack entry, this access is valid.
        let last_ctrl_stack_entry = &ctrl_stack[ctrl_stack.len() - 1];
        if stack.len() == last_ctrl_stack_entry.height + expected_val_types.len() {
            Ok(())
        } else {
            Err(Error::EndInvalidValueStack)
        }
    }

    /// Assert that the types retrieved from the type stack by `pop_valtype` unify to `expected_val_types`, and
    /// after this operation the type stack would be the same as the first time the current control block is entered.
    /// This method will unify the types on the stack to the expected valtypes if `unify_to_expected_types` is set.
    /// Any occurence of an error may leave the stack in an invalid state.
    ///
    /// # Returns
    ///
    /// - `Ok(_)`, the tail of the stack unifies to the `expected_val_types`
    /// - `Err(_)` otherwise
    ///
    pub(super) fn assert_val_types_on_top(
        &mut self,
        expected_val_types: &[ValType],
        unify_to_expected_types: bool,
    ) -> Result<(), Error> {
        ValidationStack::assert_val_types_on_top_with_custom_stacks(
            &mut self.stack,
            &self.ctrl_stack,
            expected_val_types,
            unify_to_expected_types,
        )
    }

    /// Assert that the types retrieved from the type stack by `pop_valtype` unify to `expected_val_types`.
    /// This method will unify the types on the stack to the expected valtypes if `unify_to_expected_types` is set.
    /// Any occurence of an error may leave the stack in an invalid state.
    ///
    /// # Returns
    ///
    /// - `Ok(_)`, the tail of the stack unifies to the `expected_val_types`
    /// - `Err(_)` otherwise
    ///
    pub fn assert_val_types(
        &mut self,
        expected_val_types: &[ValType],
        unify_to_expected_types: bool,
    ) -> Result<(), Error> {
        ValidationStack::assert_val_types_with_custom_stacks(
            &mut self.stack,
            &self.ctrl_stack,
            expected_val_types,
            unify_to_expected_types,
        )
    }

    /// Call `assert_val_types_on_top` for the label signature of the `label_idx`th outer control block (0 corresponds to the current control block).
    /// Label signature of all controi blocks are the output signature of the control blocks except for the Loop block. For Loop blocks, it is the input signature.
    /// This method will unify the types on the stack to the expected valtypes if `unify_to_expected_types` is set.
    ///
    /// # Returns
    ///
    /// - `Ok(_)`, the tail of the stack unifies to the label signature of the  `label_idx`th outer control block
    /// - `Err(_)` otherwise
    ///
    pub fn assert_val_types_of_label_jump_types_on_top(
        &mut self,
        label_idx: usize,
        unify_to_expected_types: bool,
    ) -> Result<(), Error> {
        let label_types = self
            .ctrl_stack
            .get(self.ctrl_stack.len() - label_idx - 1)
            .ok_or(Error::InvalidLabelIdx(label_idx))?
            .label_types();
        ValidationStack::assert_val_types_on_top_with_custom_stacks(
            &mut self.stack,
            &self.ctrl_stack,
            label_types,
            unify_to_expected_types,
        )
    }

    /// Signal to this struct that a new control block is entered, and calls `assert_val_types_on_top` with the input signature of the new control block.
    /// This method will unify the types on the stack to the expected valtypes if `unify_to_expected_types` is set.
    ///
    /// # Returns
    ///
    /// - `Ok(_)`, the tail of the stack unifies to the input signature of the  new control block
    /// - `Err(_)` otherwise
    ///
    pub fn assert_push_ctrl(
        &mut self,
        label_info: LabelInfo,
        block_ty: FuncType,
        unify_to_expected_types: bool,
    ) -> Result<(), Error> {
        self.assert_val_types_on_top(&block_ty.params.valtypes, unify_to_expected_types)?;
        let height = self.stack.len() - block_ty.params.valtypes.len();
        self.ctrl_stack.push(CtrlStackEntry {
            label_info,
            block_ty,
            height,
            unreachable: false,
        });
        Ok(())
    }

    /// Signal to this struct that the current control block is exited, and calls `assert_val_types_on_top` with the output signature of the new control block.
    /// This method will unify the types on the stack to the expected valtypes if `unify_to_expected_types` is set.
    ///
    /// # Returns
    ///
    /// - `Ok(_)`, the tail of the stack unifies to the output signature of the current control block
    /// - `Err(_)` otherwise
    ///
    pub fn assert_pop_ctrl(
        &mut self,
        unify_to_expected_types: bool,
    ) -> Result<(LabelInfo, FuncType), Error> {
        let return_types = &self
            .ctrl_stack
            .last()
            .ok_or(Error::ValidationCtrlStackEmpty)?
            .block_ty
            .returns
            .valtypes;
        ValidationStack::assert_val_types_with_custom_stacks(
            &mut self.stack,
            &self.ctrl_stack,
            return_types,
            unify_to_expected_types,
        )?;

        //if we can assert types in the above there is a last ctrl stack entry, this access is valid.
        let last_ctrl_stack_entry = self.ctrl_stack.pop().unwrap();
        Ok((
            last_ctrl_stack_entry.label_info,
            last_ctrl_stack_entry.block_ty,
        ))
    }

    /// Validate the `SELECT` instruction within the current control block. Returns OK(()) on success, Err(_) otherwise.
    pub fn validate_polymorphic_select(&mut self) -> Result<(), Error> {
        //SELECT instruction has the type signature
        //[t t i32] -> [t] where t unifies to a NumType(_) or VecType

        self.assert_pop_val_type(ValType::NumType(crate::NumType::I32))?;

        let first_arg = self.pop_valtype()?;
        let second_arg = self.pop_valtype()?;

        let unified_type = second_arg
            .unify(&first_arg)
            .ok_or(Error::InvalidValidationStackValType(None))?;

        // t must unify to a NumType(_) or VecType
        if !(unified_type.unifies_to(&ValidationStackEntry::Val(ValType::NumType(NumType::I32)))
            || unified_type.unifies_to(&ValidationStackEntry::Val(ValType::NumType(NumType::F32)))
            || unified_type.unifies_to(&ValidationStackEntry::Val(ValType::NumType(NumType::I64)))
            || unified_type.unifies_to(&ValidationStackEntry::Val(ValType::NumType(NumType::F64)))
            || unified_type.unifies_to(&ValidationStackEntry::Val(ValType::VecType)))
        {
            return Err(Error::InvalidValidationStackValType(None));
        }

        self.stack.push(unified_type);
        Ok(())
    }
}

/// corresponds to `opdtype` <https://webassembly.github.io/spec/core/valid/instructions.html#instructions>
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValidationStackEntry {
    Val(ValType),
    Bottom,
}

impl ValidationStackEntry {
    /// corresponds to whether `(self, other)` is a member of "matches" (<=) relation defined in <https://webassembly.github.io/spec/core/valid/instructions.html#instructions>
    fn unifies_to(&self, other: &ValidationStackEntry) -> bool {
        match self {
            ValidationStackEntry::Bottom => true,
            ValidationStackEntry::Val(_) => self == other,
        }
    }

    /// convenience method that returns `Some(other)` if `self.unifies_to(other)` is true and `None` otherwise
    fn unify(&self, other: &ValidationStackEntry) -> Option<Self> {
        self.unifies_to(other).then(|| other.clone())
    }
}

// TODO hide implementation
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CtrlStackEntry {
    pub label_info: LabelInfo,
    pub block_ty: FuncType,
    pub height: usize,
    pub unreachable: bool,
}

impl CtrlStackEntry {
    pub fn label_types(&self) -> &[ValType] {
        if matches!(self.label_info, LabelInfo::Loop { .. }) {
            &self.block_ty.params.valtypes
        } else {
            &self.block_ty.returns.valtypes
        }
    }
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
    Func {
        stps_to_backpatch: Vec<usize>,
    },
    Untyped,
}

#[cfg(test)]
mod tests {
    use crate::{NumType, RefType, ValType};

    use super::{CtrlStackEntry, FuncType, LabelInfo, ResultType, ValidationStack, Vec};

    fn push_dummy_untyped_label(validation_stack: &mut ValidationStack) {
        validation_stack.ctrl_stack.push(CtrlStackEntry {
            label_info: LabelInfo::Untyped,
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
            .assert_val_types(
                &[
                    ValType::NumType(NumType::F64),
                    ValType::NumType(NumType::I32),
                    ValType::NumType(NumType::F32),
                ],
                true,
            )
            .unwrap();

        push_dummy_untyped_label(&mut stack);

        stack.push_valtype(ValType::NumType(NumType::I32));

        stack
            .assert_val_types(&[ValType::NumType(NumType::I32)], true)
            .unwrap();
    }

    #[test]
    fn assert_emtpy_valtypes() {
        let mut stack = ValidationStack::new();

        stack.assert_val_types(&[], true).unwrap();

        stack.push_valtype(ValType::NumType(NumType::I32));
        push_dummy_untyped_label(&mut stack);

        // Valtypes separated by a label should also not be detected
        stack.assert_val_types(&[], true).unwrap();
    }

    #[test]
    fn assert_valtypes_on_top() {
        let mut stack = ValidationStack::new();

        stack.assert_val_types_on_top(&[], true).unwrap();

        stack.push_valtype(ValType::NumType(NumType::I32));
        stack.push_valtype(ValType::NumType(NumType::F32));
        stack.push_valtype(ValType::NumType(NumType::I64));

        // There are always zero valtypes on top of the stack
        stack.assert_val_types_on_top(&[], true).unwrap();

        stack
            .assert_val_types_on_top(&[ValType::NumType(NumType::I64)], true)
            .unwrap();

        stack
            .assert_val_types_on_top(
                &[
                    ValType::NumType(NumType::F32),
                    ValType::NumType(NumType::I64),
                ],
                true,
            )
            .unwrap();

        stack
            .assert_val_types_on_top(
                &[
                    ValType::NumType(NumType::I32),
                    ValType::NumType(NumType::F32),
                    ValType::NumType(NumType::I64),
                ],
                true,
            )
            .unwrap();
    }

    #[test]
    fn unspecified() {
        let mut stack = ValidationStack::new();
        push_dummy_untyped_label(&mut stack);

        stack.make_unspecified().unwrap();

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
        assert_eq!(stack.assert_val_types(&[], true), Ok(()));
    }

    #[test]
    fn unspecified2() {
        let mut stack = ValidationStack::new();
        push_dummy_untyped_label(&mut stack);

        stack.make_unspecified().unwrap();

        // Stack needs to keep track of unified types, I64 and F32 and I32 will appear.
        stack
            .assert_val_types(
                &[
                    ValType::NumType(NumType::I64),
                    ValType::NumType(NumType::F32),
                    ValType::NumType(NumType::I32),
                ],
                true,
            )
            .unwrap();

        stack.ctrl_stack.pop();

        assert_eq!(
            stack.assert_pop_val_type(ValType::NumType(NumType::I32)),
            Ok(())
        );
        assert_eq!(
            stack.assert_pop_val_type(ValType::NumType(NumType::F32)),
            Ok(())
        );
        assert_eq!(
            stack.assert_pop_val_type(ValType::NumType(NumType::I64)),
            Ok(())
        );
    }

    #[test]
    fn unspecified3() {
        let mut stack = ValidationStack::new();
        push_dummy_untyped_label(&mut stack);

        stack.make_unspecified().unwrap();

        stack.push_valtype(ValType::NumType(NumType::I32));

        // Stack needs to keep track of unified types, I64 and F32 will appear under I32.
        // Stack needs to keep track of unified types, I64 and F32 and I32 will appear.
        stack
            .assert_val_types(
                &[
                    ValType::NumType(NumType::I64),
                    ValType::NumType(NumType::F32),
                    ValType::NumType(NumType::I32),
                ],
                true,
            )
            .unwrap();

        stack.ctrl_stack.pop();

        assert_eq!(
            stack.assert_pop_val_type(ValType::NumType(NumType::I32)),
            Ok(())
        );
        assert_eq!(
            stack.assert_pop_val_type(ValType::NumType(NumType::F32)),
            Ok(())
        );
        assert_eq!(
            stack.assert_pop_val_type(ValType::NumType(NumType::I64)),
            Ok(())
        );
    }

    #[test]
    fn unspecified4() {
        let mut stack = ValidationStack::new();

        stack.push_valtype(ValType::VecType);
        stack.push_valtype(ValType::NumType(NumType::I32));

        push_dummy_untyped_label(&mut stack);

        stack.make_unspecified().unwrap();

        stack.push_valtype(ValType::VecType);
        stack.push_valtype(ValType::RefType(RefType::FuncRef));

        // Stack needs to keep track of unified types, I64 and F32 will appear below VecType and RefType
        // and above I32 and VecType
        stack
            .assert_val_types(
                &[
                    ValType::NumType(NumType::I64),
                    ValType::NumType(NumType::F32),
                    ValType::VecType,
                    ValType::RefType(RefType::FuncRef),
                ],
                true,
            )
            .unwrap();

        stack.ctrl_stack.pop();

        assert_eq!(
            stack.assert_pop_val_type(ValType::RefType(RefType::FuncRef)),
            Ok(())
        );
        assert_eq!(stack.assert_pop_val_type(ValType::VecType), Ok(()));
        assert_eq!(
            stack.assert_pop_val_type(ValType::NumType(NumType::F32)),
            Ok(())
        );
        assert_eq!(
            stack.assert_pop_val_type(ValType::NumType(NumType::I64)),
            Ok(())
        );
        assert_eq!(
            stack.assert_pop_val_type(ValType::NumType(NumType::I32)),
            Ok(())
        );
        assert_eq!(stack.assert_pop_val_type(ValType::VecType), Ok(()));
    }
}
