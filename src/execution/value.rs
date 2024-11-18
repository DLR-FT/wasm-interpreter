use alloc::vec;
use alloc::vec::Vec;
use core::f32;
use core::fmt::{Debug, Display};
use core::ops::{Add, Div, Mul, Sub};

use crate::core::reader::types::{NumType, ValType};
use crate::execution::assert_validated::UnwrapValidatedExt;
use crate::unreachable_validated;

#[derive(Clone, Debug, Copy, PartialOrd)]
pub struct F32(pub f32);

impl Display for F32 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartialEq for F32 {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Add for F32 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for F32 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Mul for F32 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl Div for F32 {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl F32 {
    pub fn abs(&self) -> Self {
        Self(f32::from_bits(self.0.to_bits() & !(1 << 31)))
    }
    pub fn neg(&self) -> Self {
        Self(f32::from_bits(self.0.to_bits() ^ (1 << 31)))
    }
    pub fn ceil(&self) -> Self {
        Self(libm::ceilf(self.0))
    }
    pub fn floor(&self) -> Self {
        Self(libm::floorf(self.0))
    }
    pub fn trunc(&self) -> Self {
        Self(libm::truncf(self.0))
    }
    pub fn round(&self) -> Self {
        Self(libm::roundf(self.0))
    }
    pub fn sqrt(&self) -> Self {
        Self(libm::sqrtf(self.0))
    }

    pub fn min(&self, rhs: Self) -> Self {
        Self(if self.0.is_nan() {
            self.0
        } else if rhs.0.is_nan() {
            rhs.0
        } else {
            self.0.min(rhs.0)
        })
    }
    pub fn max(&self, rhs: Self) -> Self {
        Self(if self.0.is_nan() {
            self.0
        } else if rhs.0.is_nan() {
            rhs.0
        } else {
            self.0.max(rhs.0)
        })
    }
    pub fn copysign(&self, rhs: Self) -> Self {
        Self(libm::copysignf(self.0, rhs.0))
    }
    pub fn from_bits(other: u32) -> Self {
        Self(f32::from_bits(other))
    }
    pub fn is_nan(&self) -> bool {
        self.0.is_nan()
    }
    pub fn is_infinity(&self) -> bool {
        self.0.is_infinite()
    }
    pub fn is_negative_infinity(&self) -> bool {
        self.0.is_infinite() && self.0 < 0.0
    }

    pub fn as_i32(&self) -> i32 {
        self.0 as i32
    }
    pub fn as_u32(&self) -> u32 {
        self.0 as u32
    }
    pub fn as_i64(&self) -> i64 {
        self.0 as i64
    }
    pub fn as_u64(&self) -> u64 {
        self.0 as u64
    }
    pub fn as_f32(&self) -> F64 {
        F64(self.0 as f64)
    }
    pub fn reinterpret_as_i32(&self) -> i32 {
        self.0.to_bits() as i32
    }
}

#[derive(Clone, Debug, Copy, PartialOrd)]
pub struct F64(pub f64);

impl Display for F64 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartialEq for F64 {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Add for F64 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for F64 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Mul for F64 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl Div for F64 {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl F64 {
    pub fn abs(&self) -> Self {
        Self(f64::from_bits(self.0.to_bits() & !(1 << 63)))
    }
    pub fn neg(&self) -> Self {
        Self(f64::from_bits(self.0.to_bits() ^ (1 << 63)))
    }
    pub fn ceil(&self) -> Self {
        Self(libm::ceil(self.0))
    }
    pub fn floor(&self) -> Self {
        Self(libm::floor(self.0))
    }
    pub fn trunc(&self) -> Self {
        Self(libm::trunc(self.0))
    }
    pub fn round(&self) -> Self {
        Self(libm::round(self.0))
    }
    pub fn sqrt(&self) -> Self {
        Self(libm::sqrt(self.0))
    }

    pub fn min(&self, rhs: Self) -> Self {
        Self(if self.0.is_nan() {
            self.0
        } else if rhs.0.is_nan() {
            rhs.0
        } else {
            self.0.min(rhs.0)
        })
    }
    pub fn max(&self, rhs: Self) -> Self {
        Self(if self.0.is_nan() {
            self.0
        } else if rhs.0.is_nan() {
            rhs.0
        } else {
            self.0.max(rhs.0)
        })
    }
    pub fn copysign(&self, rhs: Self) -> Self {
        Self(libm::copysign(self.0, rhs.0))
    }

    pub fn from_bits(other: u64) -> Self {
        Self(f64::from_bits(other))
    }
    pub fn is_nan(&self) -> bool {
        self.0.is_nan()
    }
    pub fn is_infinity(&self) -> bool {
        self.0.is_infinite()
    }
    pub fn is_negative_infinity(&self) -> bool {
        self.0.is_infinite() && self.0 < 0.0
    }

    pub fn as_i32(&self) -> i32 {
        self.0 as i32
    }
    pub fn as_u32(&self) -> u32 {
        self.0 as u32
    }
    pub fn as_i64(&self) -> i64 {
        self.0 as i64
    }
    pub fn as_u64(&self) -> u64 {
        self.0 as u64
    }
    pub fn as_f32(&self) -> F32 {
        F32(self.0 as f32)
    }
    pub fn reinterpret_as_i64(&self) -> i64 {
        self.0.to_bits() as i64
    }
}

/// A value at runtime. This is essentially a duplicate of [ValType] just with additional values.
///
/// See <https://webassembly.github.io/spec/core/exec/runtime.html#values>
// TODO implement missing variants
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    I32(u32),
    I64(u64),
    F32(F32),
    F64(F64),
    // F64,
    // V128,
}

#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)]
pub enum Ref {
    Null,
    // Func,
    // Extern,
}

impl Value {
    pub fn default_from_ty(ty: ValType) -> Self {
        match ty {
            ValType::NumType(NumType::I32) => Self::I32(0),
            ValType::NumType(NumType::I64) => Self::I64(0),
            ValType::NumType(NumType::F32) => Self::F32(F32(0.0)),
            ValType::NumType(NumType::F64) => Self::F64(F64(0.0_f64)),
            other => {
                todo!("cannot determine type for {other:?} because this value is not supported yet")
            }
        }
    }

    pub fn to_ty(&self) -> ValType {
        match self {
            Value::I32(_) => ValType::NumType(NumType::I32),
            Value::I64(_) => ValType::NumType(NumType::I64),
            Value::F32(_) => ValType::NumType(NumType::F32),
            Value::F64(_) => ValType::NumType(NumType::F64),
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

// TODO: don't let this like this, use a macro
impl From<f32> for Value {
    fn from(x: f32) -> Self {
        F32(x).into_value()
    }
}

impl From<Value> for f32 {
    fn from(value: Value) -> Self {
        F32::from(value).0
    }
}

// TODO: don't let this like this, use a macro
impl From<f64> for Value {
    fn from(x: f64) -> Self {
        F64(x).into_value()
    }
}

impl From<Value> for f64 {
    fn from(value: Value) -> Self {
        F64::from(value).0
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
impl_value_conversion!(u64);
impl_value_conversion!(i64);
impl_value_conversion!(F32);
impl_value_conversion!(F64);
