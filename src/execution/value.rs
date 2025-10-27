use core::fmt::{Debug, Display};
use core::ops::{Add, Div, Mul, Sub};
use core::{f32, f64};

use crate::addrs::FuncAddr;
use crate::core::reader::types::{NumType, ValType};
use crate::RefType;

#[derive(Clone, Debug, Copy, PartialOrd)]
#[repr(transparent)]
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
        Self(libm::fabsf(self.0))
    }
    pub fn neg(&self) -> Self {
        Self(-self.0)
    }
    pub fn ceil(&self) -> Self {
        if self.0.is_nan() {
            return Self(f32::NAN);
        }
        Self(libm::ceilf(self.0))
    }
    pub fn floor(&self) -> Self {
        if self.0.is_nan() {
            return Self(f32::NAN);
        }
        Self(libm::floorf(self.0))
    }
    pub fn trunc(&self) -> Self {
        if self.0.is_nan() {
            return Self(f32::NAN);
        }
        Self(libm::truncf(self.0))
    }
    pub fn nearest(&self) -> Self {
        if self.0.is_nan() {
            return Self(f32::NAN);
        }
        Self(libm::rintf(self.0))
    }
    pub fn round(&self) -> Self {
        Self(libm::roundf(self.0))
    }
    pub fn sqrt(&self) -> Self {
        Self(libm::sqrtf(self.0))
    }

    pub fn min(&self, rhs: Self) -> Self {
        Self(if self.0.is_nan() || rhs.0.is_nan() {
            f32::NAN
        } else if self.0 == 0.0 && rhs.0 == 0.0 {
            if self.to_bits() >> 31 == 1 {
                self.0
            } else {
                rhs.0
            }
        } else {
            self.0.min(rhs.0)
        })
    }
    pub fn max(&self, rhs: Self) -> Self {
        Self(if self.0.is_nan() || rhs.0.is_nan() {
            f32::NAN
        } else if self.0 == 0.0 && rhs.0 == 0.0 {
            if self.to_bits() >> 31 == 1 {
                rhs.0
            } else {
                self.0
            }
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
    pub fn as_f64(&self) -> F64 {
        F64(self.0 as f64)
    }
    pub fn reinterpret_as_i32(&self) -> i32 {
        self.0.to_bits() as i32
    }
    pub fn to_bits(&self) -> u32 {
        self.0.to_bits()
    }
}

#[derive(Clone, Debug, Copy, PartialOrd)]
#[repr(transparent)]
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
        Self(libm::fabs(self.0))
    }
    pub fn neg(&self) -> Self {
        Self(-self.0)
    }
    pub fn ceil(&self) -> Self {
        if self.0.is_nan() {
            return Self(f64::NAN);
        }
        Self(libm::ceil(self.0))
    }
    pub fn floor(&self) -> Self {
        if self.0.is_nan() {
            return Self(f64::NAN);
        }
        Self(libm::floor(self.0))
    }
    pub fn trunc(&self) -> Self {
        if self.0.is_nan() {
            return Self(f64::NAN);
        }
        Self(libm::trunc(self.0))
    }
    pub fn nearest(&self) -> Self {
        if self.0.is_nan() {
            return Self(f64::NAN);
        }
        Self(libm::rint(self.0))
    }
    pub fn round(&self) -> Self {
        Self(libm::round(self.0))
    }
    pub fn sqrt(&self) -> Self {
        Self(libm::sqrt(self.0))
    }

    pub fn min(&self, rhs: Self) -> Self {
        Self(if self.0.is_nan() || rhs.0.is_nan() {
            f64::NAN
        } else if self.0 == 0.0 && rhs.0 == 0.0 {
            if self.to_bits() >> 63 == 1 {
                self.0
            } else {
                rhs.0
            }
        } else {
            self.0.min(rhs.0)
        })
    }
    pub fn max(&self, rhs: Self) -> Self {
        Self(if self.0.is_nan() || rhs.0.is_nan() {
            f64::NAN
        } else if self.0 == 0.0 && rhs.0 == 0.0 {
            if self.to_bits() >> 63 == 1 {
                rhs.0
            } else {
                self.0
            }
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
    pub fn to_bits(&self) -> u64 {
        self.0.to_bits()
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
    V128([u8; 16]),
    Ref(Ref),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ref {
    Null(RefType),
    Func(FuncAddr),
    Extern(ExternAddr),
}

impl Display for Ref {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Ref::Func(func_addr) => write!(f, "FuncRef({func_addr:?})"),
            Ref::Extern(extern_addr) => write!(f, "ExternRef({extern_addr:?})"),
            Ref::Null(ty) => write!(f, "Null({ty:?})"),
        }
    }
}

impl Ref {
    pub fn ty(self) -> RefType {
        match self {
            Ref::Null(ref_type) => ref_type,
            Ref::Func(_) => RefType::FuncRef,
            Ref::Extern(_) => RefType::ExternRef,
        }
    }
}

/// The WebAssembly specification defines an externaddr as an address to an
/// "external" type, i.e. is a type which is managed by the embedder. For this
/// interpreter the task of managing external objects and relating them to
/// addresses is handed off to the user, which means that an [`ExternAddr`] can
/// simply be seen as an integer that is opaque to Wasm code without any meaning
/// assigned to it.
///
/// See: WebAssembly Specification 2.0 - 2.3.3, 4.2.1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExternAddr(pub usize);

impl Value {
    pub fn default_from_ty(ty: ValType) -> Self {
        match ty {
            ValType::NumType(NumType::I32) => Self::I32(0),
            ValType::NumType(NumType::I64) => Self::I64(0),
            ValType::NumType(NumType::F32) => Self::F32(F32(0.0)),
            ValType::NumType(NumType::F64) => Self::F64(F64(0.0_f64)),
            ValType::RefType(ref_type) => Self::Ref(Ref::Null(ref_type)),
            ValType::VecType => Self::V128([0; 16]),
        }
    }

    pub fn to_ty(&self) -> ValType {
        match self {
            Value::I32(_) => ValType::NumType(NumType::I32),
            Value::I64(_) => ValType::NumType(NumType::I64),
            Value::F32(_) => ValType::NumType(NumType::F32),
            Value::F64(_) => ValType::NumType(NumType::F64),
            Value::Ref(Ref::Null(ref_type)) => ValType::RefType(*ref_type),
            Value::Ref(Ref::Func(_)) => ValType::RefType(RefType::FuncRef),
            Value::Ref(Ref::Extern(_)) => ValType::RefType(RefType::ExternRef),
            Value::V128(_) => ValType::VecType,
        }
    }
}

/// An error used in all [`TryFrom<Value>`] implementations for Rust types ([`i32`], [`F32`], [`Ref`], ...)
#[derive(Debug, PartialEq, Eq)]
pub struct ValueTypeMismatchError;

impl Display for ValueTypeMismatchError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("failed to convert Value to a Rust value because the types did not match")
    }
}

impl From<u32> for Value {
    fn from(x: u32) -> Self {
        Value::I32(x)
    }
}
impl TryFrom<Value> for u32 {
    type Error = ValueTypeMismatchError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::I32(x) => Ok(x),
            _ => Err(ValueTypeMismatchError),
        }
    }
}

impl From<i32> for Value {
    fn from(x: i32) -> Self {
        Value::I32(x as u32)
    }
}
impl TryFrom<Value> for i32 {
    type Error = ValueTypeMismatchError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::I32(x) => Ok(x as i32),
            _ => Err(ValueTypeMismatchError),
        }
    }
}

impl From<u64> for Value {
    fn from(x: u64) -> Self {
        Value::I64(x)
    }
}
impl TryFrom<Value> for u64 {
    type Error = ValueTypeMismatchError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::I64(x) => Ok(x),
            _ => Err(ValueTypeMismatchError),
        }
    }
}
impl From<i64> for Value {
    fn from(x: i64) -> Self {
        Value::I64(x as u64)
    }
}
impl TryFrom<Value> for i64 {
    type Error = ValueTypeMismatchError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::I64(x) => Ok(x as i64),
            _ => Err(ValueTypeMismatchError),
        }
    }
}

impl From<F32> for Value {
    fn from(x: F32) -> Self {
        Value::F32(x)
    }
}
impl TryFrom<Value> for F32 {
    type Error = ValueTypeMismatchError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::F32(x) => Ok(x),
            _ => Err(ValueTypeMismatchError),
        }
    }
}

impl From<F64> for Value {
    fn from(x: F64) -> Self {
        Value::F64(x)
    }
}
impl TryFrom<Value> for F64 {
    type Error = ValueTypeMismatchError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::F64(x) => Ok(x),
            _ => Err(ValueTypeMismatchError),
        }
    }
}

impl From<[u8; 16]> for Value {
    fn from(value: [u8; 16]) -> Self {
        Value::V128(value)
    }
}
impl TryFrom<Value> for [u8; 16] {
    type Error = ValueTypeMismatchError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::V128(x) => Ok(x),
            _ => Err(ValueTypeMismatchError),
        }
    }
}

impl From<Ref> for Value {
    fn from(value: Ref) -> Self {
        Self::Ref(value)
    }
}

impl TryFrom<Value> for Ref {
    type Error = ValueTypeMismatchError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Ref(rref) => Ok(rref),
            _ => Err(ValueTypeMismatchError),
        }
    }
}

#[cfg(test)]
mod test {
    use alloc::string::ToString;

    use crate::{
        addrs::Addr,
        value::{ExternAddr, F32, F64},
        RefType,
    };

    use super::{FuncAddr, Ref};

    #[test]
    fn rounding_f32() {
        let round_towards_0_f32 = F32(0.5 - f32::EPSILON).round();
        let round_towards_1_f32 = F32(0.5 + f32::EPSILON).round();

        assert_eq!(round_towards_0_f32, F32(0.0));
        assert_eq!(round_towards_1_f32, F32(1.0));
    }

    #[test]
    fn rounding_f64() {
        let round_towards_0_f64 = F64(0.5 - f64::EPSILON).round();
        let round_towards_1_f64 = F64(0.5 + f64::EPSILON).round();

        assert_eq!(round_towards_0_f64, F64(0.0));
        assert_eq!(round_towards_1_f64, F64(1.0));
    }

    #[test]
    fn display_f32() {
        for x in [
            -1.0,
            0.0,
            1.0,
            13.3,
            f32::INFINITY,
            f32::MAX,
            f32::MIN,
            f32::NAN,
            f32::NEG_INFINITY,
            core::f32::consts::PI,
        ] {
            let wrapped = F32(x).to_string();
            let expected = x.to_string();
            assert_eq!(wrapped, expected);
        }
    }

    #[test]
    fn display_f64() {
        for x in [
            -1.0,
            0.0,
            1.0,
            13.3,
            f64::INFINITY,
            f64::MAX,
            f64::MIN,
            f64::NAN,
            f64::NEG_INFINITY,
            core::f64::consts::PI,
        ] {
            let wrapped = F64(x).to_string();
            let expected = x.to_string();
            assert_eq!(wrapped, expected);
        }
    }

    #[test]
    fn display_ref() {
        assert_eq!(
            Ref::Func(FuncAddr::new_unchecked(11)).to_string(),
            "FuncRef(FuncAddr(11))"
        );
        assert_eq!(
            Ref::Extern(ExternAddr(13)).to_string(),
            "ExternRef(ExternAddr(13))"
        );
        assert_eq!(Ref::Null(RefType::FuncRef).to_string(), "Null(FuncRef)");
        assert_eq!(Ref::Null(RefType::ExternRef).to_string(), "Null(ExternRef)");
    }
}
