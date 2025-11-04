use crate::{
    addrs::FuncAddr,
    checked::Stored,
    value::{ExternAddr, ValueTypeMismatchError, F32, F64},
    RefType,
};

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

/// A stored variant of [`Ref`]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StoredRef {
    Null(RefType),
    Func(Stored<FuncAddr>),
    Extern(Stored<ExternAddr>),
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
