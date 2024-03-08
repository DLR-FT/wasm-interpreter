use crate::core::reader::types::ValType;
use crate::execution::label::{ActivationFrame, Label};
use crate::execution::unwrap_validated::UnwrapValidatedExt;
use crate::execution::value::Value;
use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::unreachable_validated;
/// The stack at runtime containing
/// 1. Values
/// 2. Labels
/// 3. Activations
///
/// See <https://webassembly.github.io/spec/core/exec/runtime.html#stack>
#[derive(Clone, Debug, Default)]
pub(crate) struct Stack {
    inner: Vec<StackEntry>,
}

impl Stack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pop_value(&mut self, ty: ValType) -> Value {
        let popped = self.inner.pop().unwrap_validated();
        match popped {
            StackEntry::Value(popped) => {
                if popped.to_ty() == ty {
                    popped
                } else {
                    unreachable_validated!()
                }
            }
            StackEntry::Label(_) => todo!("pop label from stack"),
            StackEntry::Activation(_) => todo!("pop activation frame from stack"),
        }
    }

    pub fn push_value(&mut self, value: Value) {
        self.inner.push(StackEntry::Value(value));
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

/// This is a temporary solution for storing data on the stack.
///
/// In the future this will be replaced by storing raw bytes on the stack.
/// Also the enum type tag will completely removed because the types popped are always correct due to prior validation.
// TODO implement missing variants
#[derive(Clone, Debug)]
pub enum StackEntry {
    Value(Value),
    Label(Label),
    Activation(ActivationFrame),
}

impl StackEntry {
    /// Assumes this entry to be a [Value]
    pub fn into_value(self) -> Value {
        match self {
            StackEntry::Value(v) => v,
            _ => unreachable_validated!(),
        }
    }

    /// Assumes this entry to be a [Label]
    pub fn into_label(self) -> Label {
        match self {
            StackEntry::Label(l) => l,
            _ => unreachable_validated!(),
        }
    }

    /// Assumes this entry to be an [ActivationFrame]
    pub fn into_activation(self) -> ActivationFrame {
        match self {
            StackEntry::Activation(a) => a,
            _ => unreachable_validated!(),
        }
    }
}
