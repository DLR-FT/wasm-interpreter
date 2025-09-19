use crate::{
    assert_validated::UnwrapValidatedExt,
    unreachable_validated,
    value::{ExternAddr, FuncAddr, Ref, F32, F64},
    NumType, RefType, ValType, Value,
};

use alloc::{fmt::Debug, vec, vec::Vec};

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

impl InteropValue for F32 {
    const TY: ValType = ValType::NumType(NumType::F32);

    #[allow(warnings)]
    fn into_value(self) -> Value {
        Value::F32(F32(f32::from_le_bytes(self.0.to_le_bytes())))
    }

    #[allow(warnings)]
    fn from_value(value: Value) -> Self {
        match value {
            Value::F32(f) => F32(f32::from_le_bytes(f.0.to_le_bytes())),
            _ => unreachable_validated!(),
        }
    }
}

impl InteropValue for f32 {
    const TY: ValType = ValType::NumType(NumType::F32);

    #[allow(warnings)]
    fn into_value(self) -> Value {
        Value::F32(F32(f32::from_le_bytes(self.to_le_bytes())))
    }

    #[allow(warnings)]
    fn from_value(value: Value) -> Self {
        match value {
            Value::F32(f) => f32::from_le_bytes(f.0.to_le_bytes()),
            _ => unreachable_validated!(),
        }
    }
}

impl InteropValue for F64 {
    const TY: ValType = ValType::NumType(NumType::F64);

    #[allow(warnings)]
    fn into_value(self) -> Value {
        Value::F64(F64(f64::from_le_bytes(self.0.to_le_bytes())))
    }

    #[allow(warnings)]
    fn from_value(value: Value) -> Self {
        match value {
            Value::F64(f) => F64(f64::from_le_bytes(f.0.to_le_bytes())),
            _ => unreachable_validated!(),
        }
    }
}

impl InteropValue for f64 {
    const TY: ValType = ValType::NumType(NumType::F64);

    #[allow(warnings)]
    fn into_value(self) -> Value {
        Value::F64(F64(f64::from_le_bytes(self.to_le_bytes())))
    }

    #[allow(warnings)]
    fn from_value(value: Value) -> Self {
        match value {
            Value::F64(f) => f64::from_le_bytes(f.0.to_le_bytes()),
            _ => unreachable_validated!(),
        }
    }
}

impl InteropValue for Option<FuncAddr> {
    const TY: ValType = ValType::RefType(RefType::FuncRef);

    #[allow(warnings)]
    fn into_value(self) -> Value {
        let rref = self.map(Ref::Func).unwrap_or(Ref::Null(RefType::FuncRef));
        Value::Ref(rref)
    }

    #[allow(warnings)]
    fn from_value(value: Value) -> Self {
        match value {
            Value::Ref(Ref::Null(RefType::FuncRef)) => None,
            Value::Ref(Ref::Func(func_addr)) => Some(func_addr),
            _ => unreachable_validated!(),
        }
    }
}

impl InteropValue for Option<ExternAddr> {
    const TY: ValType = ValType::RefType(RefType::ExternRef);

    fn into_value(self) -> Value {
        let rref = self
            .map(Ref::Extern)
            .unwrap_or(Ref::Null(RefType::ExternRef));
        Value::Ref(rref)
    }

    fn from_value(value: Value) -> Self {
        match value {
            Value::Ref(Ref::Null(RefType::ExternRef)) => None,
            Value::Ref(Ref::Extern(extern_addr)) => Some(extern_addr),
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

impl From<f32> for Value {
    fn from(x: f32) -> Self {
        F32(x).into_value()
    }
}

impl TryFrom<Value> for f32 {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        F32::try_from(value).map(|value| value.0)
    }
}

impl From<f64> for Value {
    fn from(x: f64) -> Self {
        F64(x).into_value()
    }
}

impl TryFrom<Value> for f64 {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        F64::try_from(value).map(|value| value.0)
    }
}
