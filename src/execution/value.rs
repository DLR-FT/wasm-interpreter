use alloc::vec;
use alloc::vec::Vec;
use core::fmt::Debug;

use crate::core::reader::types::{NumType, ValType};
use crate::execution::assert_validated::UnwrapValidatedExt;
use crate::unreachable_validated;

/// A value at runtime. This is essentially a duplicate of [ValType] just with additional values.
///
/// See <https://webassembly.github.io/spec/core/exec/runtime.html#values>
// TODO implement missing variants
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Value {
    I32(u32),
    I64(u64),
    // F32,
    // F64,
    // V128,
}

#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)]
pub(crate) enum Ref {
    Null,
    // Func,
    // Extern,
}

impl Value {
    pub fn default_from_ty(ty: ValType) -> Self {
        match ty {
            ValType::NumType(NumType::I32) => Self::I32(0),
            ValType::NumType(NumType::I64) => Self::I64(0),
            other => {
                todo!("cannot determine type for {other:?} because this value is not supported yet")
            }
        }
    }

    pub fn to_ty(&self) -> ValType {
        match self {
            Value::I32(_) => ValType::NumType(NumType::I32),
            Value::I64(_) => ValType::NumType(NumType::I64),
        }
    }
}

// ------------------------------ INTEROP VALUE -------------------------------------

/// An [InteropValue] is a Rust types that can be converted into a WASM [Value].
/// This trait is intended to simplify translation between Rust values and WASM values and thus is not used internally.
pub trait InteropValue: Copy + Debug + PartialEq {
    // Sadly we cannot use `SIZE` to return fixed-sized arrays because this is still unstable.
    // See feature(generic_const_exprs)
    const TY: ValType;
    #[allow(warnings)]
    fn into_value(self) -> Value;
    #[allow(warnings)]
    fn from_value(value: Value) -> Self;
}

/// An [InteropValueList] is an iterable list of [InteropValue]s (i.e. Rust types that can be converted into WASM [Value]s).
pub trait InteropValueList {
    const TYS: &'static [ValType];
    #[allow(warnings)]
    fn into_values(self) -> Vec<Value>;
    #[allow(warnings)]
    fn from_values(values: impl Iterator<Item = Value>) -> Self;
}

impl InteropValue for u32 {
    const TY: ValType = ValType::NumType(NumType::I32);
    #[allow(warnings)]
    fn into_value(self) -> Value {
        Value::I32(self)
    }

    #[allow(warnings)]
    fn from_value(value: Value) -> Self {
        match value {
            Value::I32(i) => i,
            _ => unreachable_validated!(),
        }
    }
}

impl InteropValue for i32 {
    const TY: ValType = ValType::NumType(NumType::I32);

    #[allow(warnings)]
    fn into_value(self) -> Value {
        Value::I32(u32::from_le_bytes(self.to_le_bytes()))
    }

    #[allow(warnings)]
    fn from_value(value: Value) -> Self {
        match value {
            Value::I32(i) => i32::from_le_bytes(i.to_le_bytes()),
            _ => unreachable_validated!(),
        }
    }
}

impl InteropValue for u64 {
    const TY: ValType = ValType::NumType(NumType::I64);

    #[allow(warnings)]
    fn into_value(self) -> Value {
        Value::I64(self)
    }

    #[allow(warnings)]
    fn from_value(value: Value) -> Self {
        match value {
            Value::I64(i) => i,
            _ => unreachable_validated!(),
        }
    }
}

impl InteropValue for i64 {
    const TY: ValType = ValType::NumType(NumType::I64);

    #[allow(warnings)]
    fn into_value(self) -> Value {
        Value::I64(u64::from_le_bytes(self.to_le_bytes()))
    }

    #[allow(warnings)]
    fn from_value(value: Value) -> Self {
        match value {
            Value::I64(i) => i64::from_le_bytes(i.to_le_bytes()),
            _ => unreachable_validated!(),
        }
    }
}

impl InteropValueList for () {
    const TYS: &'static [ValType] = &[];

    #[allow(warnings)]
    fn into_values(self) -> Vec<Value> {
        Vec::new()
    }

    #[allow(warnings)]
    fn from_values(_values: impl Iterator<Item = Value>) -> Self {}
}

impl<A: InteropValue> InteropValueList for A {
    const TYS: &'static [ValType] = &[A::TY];

    #[allow(warnings)]
    fn into_values(self) -> Vec<Value> {
        vec![self.into_value()]
    }

    #[allow(warnings)]
    fn from_values(mut values: impl Iterator<Item = Value>) -> Self {
        A::from_value(values.next().unwrap_validated())
    }
}

impl<A: InteropValue> InteropValueList for (A,) {
    const TYS: &'static [ValType] = &[A::TY];
    #[allow(warnings)]
    fn into_values(self) -> Vec<Value> {
        vec![self.0.into_value()]
    }

    #[allow(warnings)]
    fn from_values(mut values: impl Iterator<Item = Value>) -> Self {
        (A::from_value(values.next().unwrap_validated()),)
    }
}

impl<A: InteropValue, B: InteropValue> InteropValueList for (A, B) {
    const TYS: &'static [ValType] = &[A::TY, B::TY];
    #[allow(warnings)]
    fn into_values(self) -> Vec<Value> {
        vec![self.0.into_value(), self.1.into_value()]
    }

    #[allow(warnings)]
    fn from_values(mut values: impl Iterator<Item = Value>) -> Self {
        (
            A::from_value(values.next().unwrap_validated()),
            B::from_value(values.next().unwrap_validated()),
        )
    }
}

impl<A: InteropValue, B: InteropValue, C: InteropValue> InteropValueList for (A, B, C) {
    const TYS: &'static [ValType] = &[A::TY, B::TY, C::TY];
    #[allow(warnings)]
    fn into_values(self) -> Vec<Value> {
        vec![
            self.0.into_value(),
            self.1.into_value(),
            self.2.into_value(),
        ]
    }

    #[allow(warnings)]
    fn from_values(mut values: impl Iterator<Item = Value>) -> Self {
        (
            A::from_value(values.next().unwrap_validated()),
            B::from_value(values.next().unwrap_validated()),
            C::from_value(values.next().unwrap_validated()),
        )
    }
}

/// Stupid From and Into implementations, because Rust's orphan rules won't let me define a generic impl:
macro_rules! impl_value_conversion {
    ($ty:ty) => {
        impl From<$ty> for Value {
            fn from(x: $ty) -> Self {
                x.into_value()
            }
        }
        impl From<Value> for $ty {
            fn from(value: Value) -> Self {
                <$ty>::from_value(value)
            }
        }
    };
}

impl_value_conversion!(u32);
impl_value_conversion!(i32);
