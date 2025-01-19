use super::Result;
use alloc::vec;
use alloc::vec::Vec;

use crate::{
    core::reader::types::{FuncType, ResultType},
    Error, RefType, ValType,
};

#[derive(Debug, PartialEq, Eq)]
pub struct ValidationStack {
    stack: Vec<ValidationStackEntry>,
    // TODO hide implementation
    pub ctrl_stack: Vec<CtrlStackEntry>,
}

impl ValidationStack {
    /// Initialize a new ValidationStack
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

    /// DANGER! only to be used within const validation! use within non-const validation may result in algorithmically incorrect validation
    pub fn peek_const_validation_stack(&self) -> Option<ValidationStackEntry> {
        self.stack.last().cloned()
    }

    /// Similar to [`ValidationStack::pop_valtype`], because it pops a value from the stack,
    /// but more public and doesn't actually return the popped value.
    pub(super) fn drop_val(&mut self) -> Result<()> {
        self.pop_valtype().map_err(|_| Error::ExpectedAnOperand)?;
        Ok(())
    }

    pub(super) fn make_unspecified(&mut self) -> Result<()> {
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
    ///   at least one element.
    /// - Returns `Err(_)` if the stack was already empty.
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
            self.stack.pop().ok_or(Error::EndInvalidValueStack)
        }
    }

    pub fn assert_pop_ref_type(&mut self, expected_ty: Option<RefType>) -> Result<()> {
        match self.pop_valtype()? {
            ValidationStackEntry::Val(ValType::RefType(ref_type)) => {
                expected_ty.map_or(Ok(()), |ty| {
                    (ty == ref_type)
                        .then_some(())
                        .ok_or(Error::DifferentRefTypes(ref_type, ty))
                })
            }
            ValidationStackEntry::Val(v) => Err(Error::ExpectedARefType(v)),
            // TODO fix the thrown error type below
            ValidationStackEntry::NumOrVecType => Err(Error::EndInvalidValueStack),
            ValidationStackEntry::UnspecifiedValTypes => Ok(()),
        }
    }

    /// Assert the top-most [`ValidationStackEntry`] is a specific [`ValType`], after popping it from the [`ValidationStack`]
    /// This assertion will unify the the top-most entry with `expected_ty`.
    ///
    /// # Returns
    ///
    /// - Returns `Ok(())` if the top-most [`ValidationStackEntry`] is a [`ValType`] identical to
    ///   `expected_ty`.
    /// - Returns `Err(_)` otherwise.
    ///
    pub fn assert_pop_val_type(&mut self, expected_ty: ValType) -> Result<()> {
        match self.pop_valtype()? {
            ValidationStackEntry::Val(ty) => (ty == expected_ty)
                .then_some(())
                .ok_or(Error::InvalidValidationStackValType(Some(ty))),
            ValidationStackEntry::NumOrVecType => match expected_ty {
                ValType::NumType(_) => Ok(()),
                ValType::VecType => Ok(()),
                // TODO change this error
                _ => Err(Error::InvalidValidationStackValType(None)),
            },
            ValidationStackEntry::UnspecifiedValTypes => Ok(()),
        }
    }

    // private fns to shut the borrow checker up when calling methods with mutable ref to self with immutable ref to self arguments
    // TODO ugly but I can't come up with anything else better

    fn assert_val_types_on_top_with_custom_stacks(
        stack: &mut Vec<ValidationStackEntry>,
        ctrl_stack: &[CtrlStackEntry],
        expected_val_types: &[ValType],
    ) -> Result<()> {
        let last_ctrl_stack_entry = ctrl_stack.last().ok_or(Error::ValidationCtrlStackEmpty)?;
        let stack_len = stack.len();

        let rev_iterator = expected_val_types.iter().rev().enumerate();
        for (i, expected_ty) in rev_iterator {
            if stack_len - last_ctrl_stack_entry.height <= i {
                if last_ctrl_stack_entry.unreachable {
                    // Unify(t2*,expected_val_types) := [t2* expected_val_types]
                    stack.splice(
                        stack_len - i..stack_len - i,
                        expected_val_types[..expected_val_types.len() - i]
                            .iter()
                            .map(|ty| ValidationStackEntry::Val(*ty)),
                    );
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
                ValidationStackEntry::NumOrVecType => match expected_ty {
                    // unify the NumOrVecType to expected_ty
                    ValType::NumType(_) => *actual_ty = ValidationStackEntry::Val(*expected_ty),
                    ValType::VecType => *actual_ty = ValidationStackEntry::Val(*expected_ty),
                    _ => return Err(Error::EndInvalidValueStack),
                },
                ValidationStackEntry::UnspecifiedValTypes => {
                    unreachable!("bottom type should not exist in the stack")
                }
            }
        }

        Ok(())
    }

    fn assert_val_types_with_custom_stacks(
        stack: &mut Vec<ValidationStackEntry>,
        ctrl_stack: &[CtrlStackEntry],
        expected_val_types: &[ValType],
    ) -> Result<()> {
        ValidationStack::assert_val_types_on_top_with_custom_stacks(
            stack,
            ctrl_stack,
            expected_val_types,
        )?;
        //if we can assert types in the above there is a last ctrl stack entry, this access is valid.
        let last_ctrl_stack_entry = &ctrl_stack[ctrl_stack.len() - 1];
        if stack.len() == last_ctrl_stack_entry.height + expected_val_types.len() {
            Ok(())
        } else {
            Err(Error::EndInvalidValueStack)
        }
    }
    /// Asserts that the values on top of the stack match those of a value iterator
    /// This method will unify the types on the stack to the expected valtypes.
    /// The last element of `expected_val_types` is unified to the top-most
    /// [`ValidationStackEntry`], the second last `expected_val_types` element to the second top-most
    /// [`ValidationStackEntry`] etc.
    ///
    /// Any unification failure or arity mismatch will cause an error.
    ///
    /// Any occurence of an error may leave the stack in an invalid state.
    ///
    /// # Returns
    ///
    /// - `Ok(_)`, the tail of the stack matches the `expected_val_types`
    /// - `Err(_)` otherwise
    ///
    pub(super) fn assert_val_types_on_top(&mut self, expected_val_types: &[ValType]) -> Result<()> {
        ValidationStack::assert_val_types_on_top_with_custom_stacks(
            &mut self.stack,
            &self.ctrl_stack,
            expected_val_types,
        )
    }

    // TODO better documentation
    /// Asserts that the valtypes on the stack match the expected valtypes and no other type is on the stack.
    /// This method will unify the types on the stack to the expected valtypes.
    /// This starts by comparing the top-most valtype with the last element from `expected_val_types` and then continues downwards on the stack.
    /// If a label is reached and not all `expected_val_types` have been checked, the assertion fails.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if all expected valtypes were found
    /// - `Err(_)` otherwise
    pub(super) fn assert_val_types(&mut self, expected_val_types: &[ValType]) -> Result<()> {
        ValidationStack::assert_val_types_with_custom_stacks(
            &mut self.stack,
            &self.ctrl_stack,
            expected_val_types,
        )
    }

    pub fn assert_val_types_of_label_jump_types_on_top(&mut self, label_idx: usize) -> Result<()> {
        let label_types = self
            .ctrl_stack
            .get(self.ctrl_stack.len() - label_idx - 1)
            .ok_or(Error::InvalidLabelIdx(label_idx))?
            .label_types();
        ValidationStack::assert_val_types_on_top_with_custom_stacks(
            &mut self.stack,
            &self.ctrl_stack,
            label_types,
        )
    }

    // TODO is moving block_ty ok?
    pub fn assert_push_ctrl(&mut self, label_info: LabelInfo, block_ty: FuncType) -> Result<()> {
        self.assert_val_types_on_top(&block_ty.params.valtypes)?;
        let height = self.stack.len() - block_ty.params.valtypes.len();
        self.ctrl_stack.push(CtrlStackEntry {
            label_info,
            block_ty,
            height,
            unreachable: false,
        });
        Ok(())
    }

    // TODO: rename/refactor this function to make it more clear that it ALSO
    // checks the stack for valid return types.
    pub fn assert_pop_ctrl(&mut self) -> Result<(LabelInfo, FuncType)> {
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
        )?;

        //if we can assert types in the above there is a last ctrl stack entry, this access is valid.
        let last_ctrl_stack_entry = self.ctrl_stack.pop().unwrap();
        Ok((
            last_ctrl_stack_entry.label_info,
            last_ctrl_stack_entry.block_ty,
        ))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValidationStackEntry {
    /// A value
    Val(ValType),
    /// Special variant to encode an uninstantiated type for `select` instruction
    #[allow(unused)]
    NumOrVecType,
    /// Special variant to encode that any possible number of [`ValType`]s could be here
    ///
    /// Caused by `return` and `unreachable`, as both can push an arbitrary number of values to the stack.
    ///
    /// When this variant is pushed onto the stack, all valtypes until the next lower label are deleted.
    /// They are not needed anymore because this variant can expand to all of them.
    // TODO change this name to BottomType
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
            .assert_val_types(&[
                ValType::NumType(NumType::F64),
                ValType::NumType(NumType::I32),
                ValType::NumType(NumType::F32),
            ])
            .unwrap();

        push_dummy_untyped_label(&mut stack);

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
        push_dummy_untyped_label(&mut stack);

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
        assert_eq!(stack.assert_val_types(&[]), Ok(()));
    }

    #[test]
    fn unspecified2() {
        let mut stack = ValidationStack::new();
        push_dummy_untyped_label(&mut stack);

        stack.make_unspecified().unwrap();

        // Stack needs to keep track of unified types, I64 and F32 and I32 will appear.
        stack
            .assert_val_types(&[
                ValType::NumType(NumType::I64),
                ValType::NumType(NumType::F32),
                ValType::NumType(NumType::I32),
            ])
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
            .assert_val_types(&[
                ValType::NumType(NumType::I64),
                ValType::NumType(NumType::F32),
                ValType::NumType(NumType::I32),
            ])
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
            .assert_val_types(&[
                ValType::NumType(NumType::I64),
                ValType::NumType(NumType::F32),
                ValType::VecType,
                ValType::RefType(RefType::FuncRef),
            ])
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
