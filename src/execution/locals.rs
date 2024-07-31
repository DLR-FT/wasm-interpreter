use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::core::reader::types::ValType;
use crate::execution::assert_validated::UnwrapValidatedExt;
use crate::execution::value::Value;

/// A helper for managing values of locals (and parameters) during function execution.
///
/// Note: As of now this stores the [Value]s. In the future storing the raw bytes without information
/// about a value's type may be preferred to minimize memory usage.
pub struct Locals {
    data: Box<[Value]>,
}

impl Locals {
    pub fn new(
        parameters: impl Iterator<Item = Value>,
        locals: impl Iterator<Item = ValType>,
    ) -> Self {
        let data = parameters
            .chain(locals.map(Value::default_from_ty))
            .collect::<Vec<Value>>()
            .into_boxed_slice();

        Self { data }
    }

    pub fn get(&self, idx: usize) -> &Value {
        self.data.get(idx).unwrap_validated()
    }

    pub fn get_ty(&self, idx: usize) -> ValType {
        self.data.get(idx).unwrap_validated().to_ty()
    }

    pub fn get_mut(&mut self, idx: usize) -> &mut Value {
        self.data.get_mut(idx).unwrap_validated()
    }
}
