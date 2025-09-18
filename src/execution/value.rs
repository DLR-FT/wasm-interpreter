use core::fmt::{Debug, Display};
use core::ops::{Add, Div, Mul, Sub};
use core::{f32, f64};

use crate::core::reader::types::{NumType, ValType};
use crate::RefType;

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
    // F64,
    // V128,
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

/// Represents the address of a function within a WebAssembly module.
///
/// Functions in WebAssembly modules can be either:
/// - **Defined**: Declared and implemented within the module.
/// - **Imported**: Declared in the module but implemented externally.
///
/// [`FuncAddr`] provides a unified representation for both types. Internally,
/// the address corresponds to an index in a combined function namespace,
/// typically represented as a vector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FuncAddr(pub usize);

/// Represents the address of an external reference in the interpreter.
///
/// External references are managed at the interpreter level and are not part of
/// the WebAssembly module itself. They are typically used to refer to host
/// functions or objects that interact with the module.
///
/// Internally, [`ExternAddr`] corresponds to an index in a linear vector,
/// enabling dynamic storage and retrieval of external values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExternAddr(pub usize);

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RefValueTy {
    Func,
    Extern,
}

impl Value {
    pub fn default_from_ty(ty: ValType) -> Self {
        match ty {
            ValType::NumType(NumType::I32) => Self::I32(0),
            ValType::NumType(NumType::I64) => Self::I64(0),
            ValType::NumType(NumType::F32) => Self::F32(F32(0.0)),
            ValType::NumType(NumType::F64) => Self::F64(F64(0.0_f64)),
            ValType::RefType(ref_type) => Self::Ref(Ref::Null(ref_type)),
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
            Value::Ref(Ref::Null(ref_type)) => ValType::RefType(*ref_type),
            Value::Ref(Ref::Func(_)) => ValType::RefType(RefType::FuncRef),
            Value::Ref(Ref::Extern(_)) => ValType::RefType(RefType::ExternRef),
        }
    }
}

impl From<u32> for Value {
    fn from(x: u32) -> Self {
        Value::I32(x)
    }
}
impl TryFrom<Value> for u32 {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::I32(x) => Ok(x),
            _ => Err(()),
        }
    }
}

impl From<i32> for Value {
    fn from(x: i32) -> Self {
        Value::I32(x as u32)
    }
}
impl TryFrom<Value> for i32 {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::I32(x) => Ok(x as i32),
            _ => Err(()),
        }
    }
}

impl From<u64> for Value {
    fn from(x: u64) -> Self {
        Value::I64(x)
    }
}
impl TryFrom<Value> for u64 {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::I64(x) => Ok(x),
            _ => Err(()),
        }
    }
}
impl From<i64> for Value {
    fn from(x: i64) -> Self {
        Value::I64(x as u64)
    }
}
impl TryFrom<Value> for i64 {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::I64(x) => Ok(x as i64),
            _ => Err(()),
        }
    }
}

impl From<F32> for Value {
    fn from(x: F32) -> Self {
        Value::F32(x)
    }
}
impl TryFrom<Value> for F32 {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::F32(x) => Ok(x),
            _ => Err(()),
        }
    }
}

impl From<F64> for Value {
    fn from(x: F64) -> Self {
        Value::F64(x)
    }
}
impl TryFrom<Value> for F64 {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::F64(x) => Ok(x),
            _ => Err(()),
        }
    }
}

impl From<Ref> for Value {
    fn from(value: Ref) -> Self {
        Self::Ref(value)
    }
}

impl TryFrom<Value> for Ref {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, ()> {
        match value {
            Value::Ref(rref) => Ok(rref),
            _ => Err(()),
        }
    }
}
