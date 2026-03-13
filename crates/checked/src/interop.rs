use alloc::{fmt::Debug, vec, vec::Vec};
use interop::{InteropValueList, RefExtern, StoreTypedInvocationExt};
use wasm::{
    addrs::FuncAddr,
    config::Config,
    value::{ValueTypeMismatchError, F32, F64},
    Hostcode, NumType, RefType, RuntimeError, ValType,
};

use crate::{stored_types::Stored, AbstractStored, Store, StoredRef, StoredValue};

/// A stored variant of [`InteropValue`](crate::execution::interop::InteropValue)
pub trait StoredInteropValue
where
    Self: Copy + Debug + PartialEq + TryFrom<StoredValue, Error = ValueTypeMismatchError>,
    StoredValue: From<Self>,
{
    const TY: ValType;
}

impl StoredInteropValue for u32 {
    const TY: ValType = ValType::NumType(NumType::I32);
}

impl StoredInteropValue for i32 {
    const TY: ValType = ValType::NumType(NumType::I32);
}

impl StoredInteropValue for u64 {
    const TY: ValType = ValType::NumType(NumType::I64);
}

impl StoredInteropValue for i64 {
    const TY: ValType = ValType::NumType(NumType::I64);
}

impl StoredInteropValue for f32 {
    const TY: ValType = ValType::NumType(NumType::F32);
}

impl StoredInteropValue for f64 {
    const TY: ValType = ValType::NumType(NumType::F64);
}

impl StoredInteropValue for [u8; 16] {
    const TY: ValType = ValType::VecType;
}

impl StoredInteropValue for StoredRefFunc {
    const TY: ValType = ValType::RefType(RefType::FuncRef);
}

impl StoredInteropValue for RefExtern {
    const TY: ValType = ValType::RefType(RefType::ExternRef);
}

impl From<f32> for StoredValue {
    fn from(value: f32) -> Self {
        F32(value).into()
    }
}

impl TryFrom<StoredValue> for f32 {
    type Error = ValueTypeMismatchError;

    fn try_from(value: StoredValue) -> Result<Self, Self::Error> {
        F32::try_from(value).map(|f| f.0)
    }
}

impl From<f64> for StoredValue {
    fn from(value: f64) -> Self {
        F64(value).into()
    }
}

impl TryFrom<StoredValue> for f64 {
    type Error = ValueTypeMismatchError;

    fn try_from(value: StoredValue) -> Result<Self, Self::Error> {
        F64::try_from(value).map(|f| f.0)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct StoredRefFunc(pub Option<Stored<FuncAddr>>);

impl From<StoredRefFunc> for StoredValue {
    fn from(value: StoredRefFunc) -> Self {
        match value.0 {
            Some(func_addr) => StoredValue::Ref(StoredRef::Func(func_addr)),
            None => StoredValue::Ref(StoredRef::Null(RefType::FuncRef)),
        }
    }
}
impl TryFrom<StoredValue> for StoredRefFunc {
    type Error = ValueTypeMismatchError;

    fn try_from(value: StoredValue) -> Result<Self, Self::Error> {
        match StoredRef::try_from(value)? {
            StoredRef::Func(func_addr) => Ok(Self(Some(func_addr))),
            StoredRef::Null(RefType::FuncRef) => Ok(Self(None)),
            _ => Err(ValueTypeMismatchError),
        }
    }
}

impl From<RefExtern> for StoredValue {
    fn from(value: RefExtern) -> Self {
        match value.0 {
            Some(extern_addr) => StoredValue::Ref(StoredRef::Extern(extern_addr)),
            None => StoredValue::Ref(StoredRef::Null(RefType::ExternRef)),
        }
    }
}

impl TryFrom<StoredValue> for RefExtern {
    type Error = ValueTypeMismatchError;

    fn try_from(value: StoredValue) -> Result<Self, Self::Error> {
        match value {
            StoredValue::Ref(StoredRef::Extern(extern_addr)) => Ok(Self(Some(extern_addr))),
            StoredValue::Ref(StoredRef::Null(RefType::ExternRef)) => Ok(Self(None)),
            _ => Err(ValueTypeMismatchError),
        }
    }
}

/// A [StoredInteropValueList] is an iterable list of [StoredInteropValue]s (i.e. Rust types that can be converted into Wasm [StoredValue]s).
pub trait StoredInteropValueList: Debug + Copy {
    const TYS: &'static [ValType];

    fn into_values(self) -> Vec<StoredValue>;

    fn try_from_values(
        values: impl ExactSizeIterator<Item = StoredValue>,
    ) -> Result<Self, ValueTypeMismatchError>;
}

impl StoredInteropValueList for () {
    const TYS: &'static [ValType] = &[];

    fn into_values(self) -> Vec<StoredValue> {
        Vec::new()
    }

    fn try_from_values(
        values: impl ExactSizeIterator<Item = StoredValue>,
    ) -> Result<Self, ValueTypeMismatchError> {
        if values.len() != 0 {
            return Err(ValueTypeMismatchError);
        }

        Ok(())
    }
}

impl<A> StoredInteropValueList for A
where
    A: StoredInteropValue,
    StoredValue: From<A>,
{
    const TYS: &'static [ValType] = &[A::TY];

    fn into_values(self) -> Vec<StoredValue> {
        vec![self.into()]
    }

    fn try_from_values(
        mut values: impl ExactSizeIterator<Item = StoredValue>,
    ) -> Result<Self, ValueTypeMismatchError> {
        if values.len() != Self::TYS.len() {
            return Err(ValueTypeMismatchError);
        }

        A::try_from(values.next().unwrap())
    }
}

impl<A> StoredInteropValueList for (A,)
where
    A: StoredInteropValue,
    StoredValue: From<A>,
{
    const TYS: &'static [ValType] = &[A::TY];

    fn into_values(self) -> Vec<StoredValue> {
        vec![self.0.into()]
    }

    fn try_from_values(
        mut values: impl ExactSizeIterator<Item = StoredValue>,
    ) -> Result<Self, ValueTypeMismatchError> {
        if values.len() != Self::TYS.len() {
            return Err(ValueTypeMismatchError);
        }

        Ok((A::try_from(values.next().unwrap())?,))
    }
}

impl<A, B> StoredInteropValueList for (A, B)
where
    A: StoredInteropValue,
    B: StoredInteropValue,
    StoredValue: From<A> + From<B>,
{
    const TYS: &'static [ValType] = &[A::TY, B::TY];

    fn into_values(self) -> Vec<StoredValue> {
        vec![self.0.into(), self.1.into()]
    }

    fn try_from_values(
        mut values: impl ExactSizeIterator<Item = StoredValue>,
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

impl<A, B, C> StoredInteropValueList for (A, B, C)
where
    A: StoredInteropValue,
    B: StoredInteropValue,
    C: StoredInteropValue,
    StoredValue: From<A> + From<B> + From<C>,
{
    const TYS: &'static [ValType] = &[A::TY, B::TY, C::TY];

    fn into_values(self) -> Vec<StoredValue> {
        vec![self.0.into(), self.1.into(), self.2.into()]
    }

    fn try_from_values(
        mut values: impl ExactSizeIterator<Item = StoredValue>,
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

impl<T: Config> Store<'_, T> {
    /// This is a variant of
    /// [`Store::func_alloc_typed_unchecked`](wasm::Store::func_alloc_typed_unchecked).
    /// It is functionally equal, with the only difference being that this
    /// function returns a [`Stored<FuncAddr>`].
    #[allow(clippy::let_and_return)] // reason = "to follow the 1234 structure"
    pub unsafe fn func_alloc_typed<Params: InteropValueList, Returns: InteropValueList>(
        &mut self,
        host_func: Hostcode,
    ) -> Stored<FuncAddr> {
        // 1. try unwrap
        // no stored parameters
        // 2. call
        let func_addr = self
            .inner
            .func_alloc_typed_unchecked::<Params, Returns>(host_func);
        // 3. rewrap
        // 4. return
        // SAFETY: The function address just came from the current store.
        unsafe { Stored::from_bare(func_addr, self.id) }
    }

    /// This is a safe variant of
    /// [`StoreTypedInvocationExt::invoke_simple_typed`]
    pub fn invoke_simple_typed<Params: StoredInteropValueList, Returns: StoredInteropValueList>(
        &mut self,
        func_addr: Stored<FuncAddr>,
        params: Params,
    ) -> Result<Returns, RuntimeError> {
        // 1. try unwrap
        let params = params.into_values().try_unwrap_into_bare(self.id);
        let func_addr = func_addr.try_unwrap_into_bare(self.id);
        // 2. call
        // Note: We cannot call the inner
        // `invoke_simple_typed` due to issues with
        // generics. Instead call the untyped version.
        // SAFETY: We just checked that the function address and all addresses
        // in the parameters are valid in the current store through their store
        // ids.
        let return_values = unsafe { self.inner.invoke_simple(func_addr, params) }?;
        // 3. rewrap
        // SAFETY: The return values just came from the current store.
        let stored_return_values = unsafe { Vec::from_bare(return_values, self.id) };
        // 4. return
        Returns::try_from_values(stored_return_values.into_iter())
            .map_err(|ValueTypeMismatchError| RuntimeError::FunctionInvocationSignatureMismatch)
    }
}
