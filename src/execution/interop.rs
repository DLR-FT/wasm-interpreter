//! This module provides types, traits and impls to convert between
//! Rust types and the Wasm [`Value`] type. Internally this module is
//! not used, except for the top-level entry points for invocation.
//!
//! The main trait is [`InteropValue`]. It is implemented for all Rust
//! types which can be converted into and from a [`Value`] through the
//! [`From`] and [`TryFrom`] traits, respectively.
//!
//! Then, the [`InteropValueList`] trait is a layer on top, allowing
//! the same conversions but instead for tuples/lists containing
//! multiple values.

use crate::{
    addrs::FuncAddr,
    value::{ExternAddr, Ref, F32, F64},
    NumType, RefType, ValType, Value,
};

use alloc::{fmt::Debug, vec, vec::Vec};

use super::value::ValueTypeMismatchError;

/// An [InteropValue] is a Rust types that can be converted into a WASM [Value].
/// This trait is intended to simplify translation between Rust values and WASM values and thus is not used internally.
pub trait InteropValue
where
    Self: Copy + Debug + PartialEq + TryFrom<Value, Error = ValueTypeMismatchError>,
    Value: From<Self>,
{
    const TY: ValType;
}

impl InteropValue for u32 {
    const TY: ValType = ValType::NumType(NumType::I32);
}

impl InteropValue for i32 {
    const TY: ValType = ValType::NumType(NumType::I32);
}

impl InteropValue for u64 {
    const TY: ValType = ValType::NumType(NumType::I64);
}

impl InteropValue for i64 {
    const TY: ValType = ValType::NumType(NumType::I64);
}

impl InteropValue for f32 {
    const TY: ValType = ValType::NumType(NumType::F32);
}

impl InteropValue for f64 {
    const TY: ValType = ValType::NumType(NumType::F64);
}

impl InteropValue for [u8; 16] {
    const TY: ValType = ValType::VecType;
}

impl InteropValue for RefFunc {
    const TY: ValType = ValType::RefType(RefType::FuncRef);
}

impl InteropValue for RefExtern {
    const TY: ValType = ValType::RefType(RefType::ExternRef);
}

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        F32(value).into()
    }
}

impl TryFrom<Value> for f32 {
    type Error = ValueTypeMismatchError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        F32::try_from(value).map(|f| f.0)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        F64(value).into()
    }
}

impl TryFrom<Value> for f64 {
    type Error = ValueTypeMismatchError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        F64::try_from(value).map(|f| f.0)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct RefFunc(pub Option<FuncAddr>);

impl From<RefFunc> for Value {
    fn from(value: RefFunc) -> Self {
        match value.0 {
            Some(func_addr) => Ref::Func(func_addr),
            None => Ref::Null(RefType::FuncRef),
        }
        .into()
    }
}

impl TryFrom<Value> for RefFunc {
    type Error = ValueTypeMismatchError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match Ref::try_from(value)? {
            Ref::Func(func_addr) => Ok(Self(Some(func_addr))),
            Ref::Null(RefType::FuncRef) => Ok(Self(None)),
            _ => Err(ValueTypeMismatchError),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct RefExtern(pub Option<ExternAddr>);

impl From<RefExtern> for Value {
    fn from(value: RefExtern) -> Self {
        match value.0 {
            Some(extern_addr) => Ref::Extern(extern_addr),
            None => Ref::Null(RefType::ExternRef),
        }
        .into()
    }
}

impl TryFrom<Value> for RefExtern {
    type Error = ValueTypeMismatchError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match Ref::try_from(value)? {
            Ref::Extern(extern_addr) => Ok(Self(Some(extern_addr))),
            Ref::Null(RefType::ExternRef) => Ok(Self(None)),
            _ => Err(ValueTypeMismatchError),
        }
    }
}

/// An [InteropValueList] is an iterable list of [InteropValue]s (i.e. Rust types that can be converted into WASM [Value]s).
pub trait InteropValueList: Debug + Copy {
    const TYS: &'static [ValType];

    fn into_values(self) -> Vec<Value>;

    fn try_from_values(
        values: impl ExactSizeIterator<Item = Value>,
    ) -> Result<Self, ValueTypeMismatchError>;
}

impl InteropValueList for () {
    const TYS: &'static [ValType] = &[];

    fn into_values(self) -> Vec<Value> {
        Vec::new()
    }

    fn try_from_values(
        values: impl ExactSizeIterator<Item = Value>,
    ) -> Result<Self, ValueTypeMismatchError> {
        if values.len() != 0 {
            return Err(ValueTypeMismatchError);
        }

        Ok(())
    }
}

impl<A> InteropValueList for A
where
    A: InteropValue,
    Value: From<A>,
{
    const TYS: &'static [ValType] = &[A::TY];

    fn into_values(self) -> Vec<Value> {
        vec![self.into()]
    }

    fn try_from_values(
        mut values: impl ExactSizeIterator<Item = Value>,
    ) -> Result<Self, ValueTypeMismatchError> {
        if values.len() != Self::TYS.len() {
            return Err(ValueTypeMismatchError);
        }

        A::try_from(values.next().unwrap())
    }
}

impl<A> InteropValueList for (A,)
where
    A: InteropValue,
    Value: From<A>,
{
    const TYS: &'static [ValType] = &[A::TY];

    fn into_values(self) -> Vec<Value> {
        vec![self.0.into()]
    }

    fn try_from_values(
        mut values: impl ExactSizeIterator<Item = Value>,
    ) -> Result<Self, ValueTypeMismatchError> {
        if values.len() != Self::TYS.len() {
            return Err(ValueTypeMismatchError);
        }

        Ok((A::try_from(values.next().unwrap())?,))
    }
}

impl<A, B> InteropValueList for (A, B)
where
    A: InteropValue,
    B: InteropValue,
    Value: From<A> + From<B>,
{
    const TYS: &'static [ValType] = &[A::TY, B::TY];

    fn into_values(self) -> Vec<Value> {
        vec![self.0.into(), self.1.into()]
    }

    fn try_from_values(
        mut values: impl ExactSizeIterator<Item = Value>,
    ) -> Result<Self, ValueTypeMismatchError> {
        if values.len() != Self::TYS.len() {
            return Err(ValueTypeMismatchError);
        }

        Ok((
            A::try_from(values.next().unwrap())?,
            B::try_from(values.next().unwrap())?,
        ))
    }
}

impl<A, B, C> InteropValueList for (A, B, C)
where
    A: InteropValue,
    B: InteropValue,
    C: InteropValue,
    Value: From<A> + From<B> + From<C>,
{
    const TYS: &'static [ValType] = &[A::TY, B::TY, C::TY];

    fn into_values(self) -> Vec<Value> {
        vec![self.0.into(), self.1.into(), self.2.into()]
    }

    fn try_from_values(
        mut values: impl ExactSizeIterator<Item = Value>,
    ) -> Result<Self, ValueTypeMismatchError> {
        if values.len() != Self::TYS.len() {
            return Err(ValueTypeMismatchError);
        }

        Ok((
            A::try_from(values.next().unwrap())?,
            B::try_from(values.next().unwrap())?,
            C::try_from(values.next().unwrap())?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::addrs::{Addr, FuncAddr};
    use crate::value::{ExternAddr, Value, ValueTypeMismatchError};
    use alloc::vec::Vec;

    use super::{InteropValueList, RefExtern, RefFunc};

    // We use function shorthands to generate error types because it's shorter
    const fn ok<T>(t: T) -> Result<T, ValueTypeMismatchError> {
        Result::<T, ValueTypeMismatchError>::Ok(t)
    }
    const fn err<T>() -> Result<T, ValueTypeMismatchError> {
        Result::<T, ValueTypeMismatchError>::Err(ValueTypeMismatchError)
    }

    #[test]
    fn roundtrip_single_u32() {
        const RUST_VALUE: u32 = 5;
        let wasm_value: Value = RUST_VALUE.into();
        assert_eq!(wasm_value.try_into(), ok(RUST_VALUE));
        assert_eq!(wasm_value.try_into(), ok(RUST_VALUE as i32));
        assert_eq!(wasm_value.try_into(), err::<u64>());
        assert_eq!(wasm_value.try_into(), err::<i64>());
        assert_eq!(wasm_value.try_into(), err::<f32>());
        assert_eq!(wasm_value.try_into(), err::<f64>());
        assert_eq!(wasm_value.try_into(), err::<RefFunc>());
        assert_eq!(wasm_value.try_into(), err::<RefExtern>());
    }

    #[test]
    fn roundtrip_single_i32() {
        const RUST_VALUE: i32 = 5;
        let wasm_value: Value = RUST_VALUE.into();
        assert_eq!(wasm_value.try_into(), ok(RUST_VALUE as u32));
        assert_eq!(wasm_value.try_into(), ok(RUST_VALUE));
        assert_eq!(wasm_value.try_into(), err::<u64>());
        assert_eq!(wasm_value.try_into(), err::<i64>());
        assert_eq!(wasm_value.try_into(), err::<f32>());
        assert_eq!(wasm_value.try_into(), err::<f64>());
        assert_eq!(wasm_value.try_into(), err::<RefFunc>());
        assert_eq!(wasm_value.try_into(), err::<RefExtern>());
    }

    #[test]
    fn roundtrip_single_u64() {
        const RUST_VALUE: u64 = 5;
        let wasm_value: Value = RUST_VALUE.into();
        assert_eq!(wasm_value.try_into(), err::<u32>());
        assert_eq!(wasm_value.try_into(), err::<i32>());
        assert_eq!(wasm_value.try_into(), ok(RUST_VALUE));
        assert_eq!(wasm_value.try_into(), ok(RUST_VALUE as i64));
        assert_eq!(wasm_value.try_into(), err::<f32>());
        assert_eq!(wasm_value.try_into(), err::<f64>());
        assert_eq!(wasm_value.try_into(), err::<RefFunc>());
        assert_eq!(wasm_value.try_into(), err::<RefExtern>());
    }

    #[test]
    fn roundtrip_single_i64() {
        const RUST_VALUE: i64 = 5;
        let wasm_value: Value = RUST_VALUE.into();
        assert_eq!(wasm_value.try_into(), err::<u32>());
        assert_eq!(wasm_value.try_into(), err::<i32>());
        assert_eq!(wasm_value.try_into(), ok(RUST_VALUE as u64));
        assert_eq!(wasm_value.try_into(), ok(RUST_VALUE));
        assert_eq!(wasm_value.try_into(), err::<f32>());
        assert_eq!(wasm_value.try_into(), err::<f64>());
        assert_eq!(wasm_value.try_into(), err::<RefFunc>());
        assert_eq!(wasm_value.try_into(), err::<RefExtern>());
    }

    #[test]
    fn roundtrip_single_f32() {
        const RUST_VALUE: f32 = 123.12;
        let wasm_value: Value = RUST_VALUE.into();
        assert_eq!(wasm_value.try_into(), err::<u32>());
        assert_eq!(wasm_value.try_into(), err::<i32>());
        assert_eq!(wasm_value.try_into(), err::<u64>());
        assert_eq!(wasm_value.try_into(), err::<i64>());
        assert_eq!(wasm_value.try_into(), ok(RUST_VALUE));
        assert_eq!(wasm_value.try_into(), err::<f64>());
        assert_eq!(wasm_value.try_into(), err::<RefFunc>());
        assert_eq!(wasm_value.try_into(), err::<RefExtern>());
    }

    #[test]
    fn roundtrip_single_f64() {
        const RUST_VALUE: f64 = 123.12;
        let wasm_value: Value = RUST_VALUE.into();
        assert_eq!(wasm_value.try_into(), err::<u32>());
        assert_eq!(wasm_value.try_into(), err::<i32>());
        assert_eq!(wasm_value.try_into(), err::<u64>());
        assert_eq!(wasm_value.try_into(), err::<i64>());
        assert_eq!(wasm_value.try_into(), err::<f32>());
        assert_eq!(wasm_value.try_into(), ok(RUST_VALUE));
        assert_eq!(wasm_value.try_into(), err::<RefFunc>());
        assert_eq!(wasm_value.try_into(), err::<RefExtern>());
    }

    #[test]
    fn roundtrip_single_ref_func() {
        let rust_value: RefFunc = RefFunc(Some(FuncAddr::new_unchecked(0)));
        let wasm_value: Value = rust_value.into();
        assert_eq!(wasm_value.try_into(), err::<u32>());
        assert_eq!(wasm_value.try_into(), err::<i32>());
        assert_eq!(wasm_value.try_into(), err::<u64>());
        assert_eq!(wasm_value.try_into(), err::<i64>());
        assert_eq!(wasm_value.try_into(), err::<f32>());
        assert_eq!(wasm_value.try_into(), err::<f64>());
        assert_eq!(wasm_value.try_into(), ok(rust_value));
        assert_eq!(wasm_value.try_into(), err::<RefExtern>());
    }

    #[test]
    fn roundtrip_single_ref_extern() {
        const RUST_VALUE: RefExtern = RefExtern(Some(ExternAddr(51)));
        let wasm_value: Value = RUST_VALUE.into();
        assert_eq!(wasm_value.try_into(), err::<u32>());
        assert_eq!(wasm_value.try_into(), err::<i32>());
        assert_eq!(wasm_value.try_into(), err::<u64>());
        assert_eq!(wasm_value.try_into(), err::<i64>());
        assert_eq!(wasm_value.try_into(), err::<f32>());
        assert_eq!(wasm_value.try_into(), err::<f64>());
        assert_eq!(wasm_value.try_into(), err::<RefFunc>());
        assert_eq!(wasm_value.try_into(), ok(RUST_VALUE));
    }

    #[test]
    fn roundtrip_single_ref_func_null() {
        const RUST_VALUE: RefFunc = RefFunc(None);
        let wasm_value: Value = RUST_VALUE.into();
        assert_eq!(wasm_value.try_into(), err::<u32>());
        assert_eq!(wasm_value.try_into(), err::<i32>());
        assert_eq!(wasm_value.try_into(), err::<u64>());
        assert_eq!(wasm_value.try_into(), err::<i64>());
        assert_eq!(wasm_value.try_into(), err::<f32>());
        assert_eq!(wasm_value.try_into(), err::<f64>());
        assert_eq!(wasm_value.try_into(), ok::<RefFunc>(RUST_VALUE));
        assert_eq!(wasm_value.try_into(), err::<RefExtern>());
    }

    #[test]
    fn roundtrip_single_ref_extern_null() {
        const RUST_VALUE: RefExtern = RefExtern(None);
        let wasm_value: Value = RUST_VALUE.into();
        assert_eq!(wasm_value.try_into(), err::<u32>());
        assert_eq!(wasm_value.try_into(), err::<i32>());
        assert_eq!(wasm_value.try_into(), err::<u64>());
        assert_eq!(wasm_value.try_into(), err::<i64>());
        assert_eq!(wasm_value.try_into(), err::<f32>());
        assert_eq!(wasm_value.try_into(), err::<f64>());
        assert_eq!(wasm_value.try_into(), err::<RefFunc>());
        assert_eq!(wasm_value.try_into(), ok::<RefExtern>(RUST_VALUE));
    }

    #[test]
    fn roundtrip_list0() {
        const RUST_VALUES: () = ();
        let wasm_values: Vec<Value> = RUST_VALUES.into_values();
        let roundtrip_rust_values = InteropValueList::try_from_values(wasm_values.into_iter());
        assert_eq!(roundtrip_rust_values, Ok(RUST_VALUES));
    }

    #[test]
    fn roundtrip_list1_single() {
        const RUST_VALUES: u32 = 5;
        let wasm_values: Vec<Value> = RUST_VALUES.into_values();
        let roundtrip_rust_values = InteropValueList::try_from_values(wasm_values.into_iter());
        assert_eq!(roundtrip_rust_values, Ok(RUST_VALUES));
    }

    #[test]
    fn roundtrip_list1() {
        const RUST_VALUES: (u32,) = (5,);
        let wasm_values: Vec<Value> = RUST_VALUES.into_values();
        let roundtrip_rust_values = InteropValueList::try_from_values(wasm_values.into_iter());
        assert_eq!(roundtrip_rust_values, Ok(RUST_VALUES));
    }

    #[test]
    fn roundtrip_list2() {
        let rust_values: (f32, RefFunc) = (3.0, RefFunc(Some(FuncAddr::new_unchecked(0))));
        let wasm_values: Vec<Value> = rust_values.into_values();
        let roundtrip_rust_values = InteropValueList::try_from_values(wasm_values.into_iter());
        assert_eq!(roundtrip_rust_values, Ok(rust_values));
    }

    #[test]
    fn roundtrip_list3() {
        const RUST_VALUES: (RefExtern, u64, i32) =
            (RefExtern(Some(ExternAddr(123))), 8472846, -61864);
        let wasm_values: Vec<Value> = RUST_VALUES.into_values();
        let roundtrip_rust_values = InteropValueList::try_from_values(wasm_values.into_iter());
        assert_eq!(roundtrip_rust_values, Ok(RUST_VALUES))
    }

    #[test]
    fn list_incorrect_lengths() {
        let wasm_values0: Vec<Value> = ().into_values();
        let wasm_values1_single: Vec<Value> = 5u32.into_values();
        let wasm_values1: Vec<Value> = (5u32,).into_values();
        let wasm_values2: Vec<Value> = (5u32, 5u32).into_values();
        let wasm_values3: Vec<Value> = (5u32, 5u32, 5u32).into_values();

        assert_eq!(
            InteropValueList::try_from_values(wasm_values0.clone().into_iter()),
            err::<u32>()
        );
        assert_eq!(
            InteropValueList::try_from_values(wasm_values0.clone().into_iter()),
            err::<(u32,)>()
        );
        assert_eq!(
            InteropValueList::try_from_values(wasm_values0.clone().into_iter()),
            err::<(u32, u32)>()
        );
        assert_eq!(
            InteropValueList::try_from_values(wasm_values0.clone().into_iter()),
            err::<(u32, u32, u32)>()
        );

        assert_eq!(
            InteropValueList::try_from_values(wasm_values1_single.clone().into_iter()),
            err::<()>()
        );
        assert_eq!(
            InteropValueList::try_from_values(wasm_values1_single.clone().into_iter()),
            err::<(u32, u32)>()
        );
        assert_eq!(
            InteropValueList::try_from_values(wasm_values1_single.clone().into_iter()),
            err::<(u32, u32, u32)>()
        );

        assert_eq!(
            InteropValueList::try_from_values(wasm_values1.clone().into_iter()),
            err::<()>()
        );
        assert_eq!(
            InteropValueList::try_from_values(wasm_values1.clone().into_iter()),
            err::<(u32, u32)>()
        );
        assert_eq!(
            InteropValueList::try_from_values(wasm_values1.clone().into_iter()),
            err::<(u32, u32, u32)>()
        );

        assert_eq!(
            InteropValueList::try_from_values(wasm_values2.clone().into_iter()),
            err::<()>()
        );
        assert_eq!(
            InteropValueList::try_from_values(wasm_values2.clone().into_iter()),
            err::<(u32,)>()
        );
        assert_eq!(
            InteropValueList::try_from_values(wasm_values2.clone().into_iter()),
            err::<(u32, u32, u32)>()
        );

        assert_eq!(
            InteropValueList::try_from_values(wasm_values3.clone().into_iter()),
            err::<()>()
        );
        assert_eq!(
            InteropValueList::try_from_values(wasm_values3.clone().into_iter()),
            err::<(u32,)>()
        );
        assert_eq!(
            InteropValueList::try_from_values(wasm_values3.clone().into_iter()),
            err::<(u32, u32)>()
        );
    }
}
