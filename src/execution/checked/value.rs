use crate::{
    addrs::FuncAddr,
    value::{ExternAddr, Ref, ValueTypeMismatchError, F32, F64},
    RefType, Value,
};

use super::{AbstractStored, Stored};

/// A stored variant of [`Value`]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StoredValue {
    I32(u32),
    I64(u64),
    F32(F32),
    F64(F64),
    V128([u8; 16]),
    Ref(StoredRef),
}

impl AbstractStored for StoredValue {
    type BareTy = Value;

    unsafe fn from_bare(bare_value: Self::BareTy, id: crate::StoreId) -> Self {
        match bare_value {
            Value::I32(x) => Self::I32(x),
            Value::I64(x) => Self::I64(x),
            Value::F32(x) => Self::F32(x),
            Value::F64(x) => Self::F64(x),
            Value::V128(x) => Self::V128(x),
            Value::Ref(r#ref) => {
                // Safety: Upheld by the caller
                let stored_ref = unsafe { StoredRef::from_bare(r#ref, id) };
                Self::Ref(stored_ref)
            }
        }
    }

    fn id(&self) -> Option<crate::StoreId> {
        match self {
            Self::Ref(r#ref) => r#ref.id(),
            _ => None,
        }
    }

    fn into_bare(self) -> Self::BareTy {
        match self {
            Self::I32(x) => Value::I32(x),
            Self::I64(x) => Value::I64(x),
            Self::F32(x) => Value::F32(x),
            Self::F64(x) => Value::F64(x),
            Self::V128(x) => Value::V128(x),
            Self::Ref(stored_ref) => Value::Ref(stored_ref.into_bare()),
        }
    }
}

/// A stored variant of [`Ref`]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StoredRef {
    Null(RefType),
    Func(Stored<FuncAddr>),
    /// We do not wrap [`ExternAddr`]s in a [`Stored`] object because they are
    /// not stored in the [`Store`](crate::Store).
    Extern(ExternAddr),
}

impl AbstractStored for StoredRef {
    type BareTy = Ref;

    unsafe fn from_bare(bare_value: Self::BareTy, id: crate::StoreId) -> Self {
        match bare_value {
            Ref::Null(ref_type) => Self::Null(ref_type),
            Ref::Func(func_addr) => {
                // Safety: Upheld by the caller
                let stored_func_addr = unsafe { Stored::from_bare(func_addr, id) };
                Self::Func(stored_func_addr)
            }
            Ref::Extern(extern_addr) => Self::Extern(extern_addr),
        }
    }

    fn id(&self) -> Option<crate::StoreId> {
        match self {
            StoredRef::Func(stored_func_addr) => stored_func_addr.id(),
            StoredRef::Null(_) | StoredRef::Extern(_) => None,
        }
    }

    fn into_bare(self) -> Self::BareTy {
        match self {
            Self::Null(ref_type) => Ref::Null(ref_type),
            Self::Func(stored_func_addr) => Ref::Func(stored_func_addr.into_bare()),
            Self::Extern(extern_addr) => Ref::Extern(extern_addr),
        }
    }
}

impl From<u32> for StoredValue {
    fn from(value: u32) -> Self {
        StoredValue::I32(value)
    }
}
impl TryFrom<StoredValue> for u32 {
    type Error = ValueTypeMismatchError;

    fn try_from(value: StoredValue) -> Result<Self, Self::Error> {
        match value {
            StoredValue::I32(value) => Ok(value),
            _ => Err(ValueTypeMismatchError),
        }
    }
}

impl From<i32> for StoredValue {
    fn from(value: i32) -> Self {
        StoredValue::I32(u32::from_le_bytes(value.to_le_bytes()))
    }
}
impl TryFrom<StoredValue> for i32 {
    type Error = ValueTypeMismatchError;

    fn try_from(value: StoredValue) -> Result<Self, Self::Error> {
        match value {
            StoredValue::I32(value) => Ok(i32::from_le_bytes(value.to_le_bytes())),
            _ => Err(ValueTypeMismatchError),
        }
    }
}

impl From<u64> for StoredValue {
    fn from(value: u64) -> Self {
        StoredValue::I64(value)
    }
}
impl TryFrom<StoredValue> for u64 {
    type Error = ValueTypeMismatchError;

    fn try_from(value: StoredValue) -> Result<Self, Self::Error> {
        match value {
            StoredValue::I64(value) => Ok(value),
            _ => Err(ValueTypeMismatchError),
        }
    }
}

impl From<i64> for StoredValue {
    fn from(value: i64) -> Self {
        StoredValue::I64(u64::from_le_bytes(value.to_le_bytes()))
    }
}
impl TryFrom<StoredValue> for i64 {
    type Error = ValueTypeMismatchError;

    fn try_from(value: StoredValue) -> Result<Self, Self::Error> {
        match value {
            StoredValue::I64(value) => Ok(i64::from_le_bytes(value.to_le_bytes())),
            _ => Err(ValueTypeMismatchError),
        }
    }
}

impl From<F32> for StoredValue {
    fn from(value: F32) -> Self {
        StoredValue::F32(value)
    }
}
impl TryFrom<StoredValue> for F32 {
    type Error = ValueTypeMismatchError;

    fn try_from(value: StoredValue) -> Result<Self, Self::Error> {
        match value {
            StoredValue::F32(value) => Ok(value),
            _ => Err(ValueTypeMismatchError),
        }
    }
}

impl From<F64> for StoredValue {
    fn from(value: F64) -> Self {
        StoredValue::F64(value)
    }
}
impl TryFrom<StoredValue> for F64 {
    type Error = ValueTypeMismatchError;

    fn try_from(value: StoredValue) -> Result<Self, Self::Error> {
        match value {
            StoredValue::F64(value) => Ok(value),
            _ => Err(ValueTypeMismatchError),
        }
    }
}

impl From<[u8; 16]> for StoredValue {
    fn from(value: [u8; 16]) -> Self {
        StoredValue::V128(value)
    }
}
impl TryFrom<StoredValue> for [u8; 16] {
    type Error = ValueTypeMismatchError;

    fn try_from(value: StoredValue) -> Result<Self, Self::Error> {
        match value {
            StoredValue::V128(value) => Ok(value),
            _ => Err(ValueTypeMismatchError),
        }
    }
}

impl From<StoredRef> for StoredValue {
    fn from(value: StoredRef) -> Self {
        StoredValue::Ref(value)
    }
}
impl TryFrom<StoredValue> for StoredRef {
    type Error = ValueTypeMismatchError;

    fn try_from(value: StoredValue) -> Result<Self, Self::Error> {
        match value {
            StoredValue::Ref(value) => Ok(value),
            _ => Err(ValueTypeMismatchError),
        }
    }
}
