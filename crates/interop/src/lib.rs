//! This module provides types, traits and impls to convert between Rust types
//! and the Wasm [`Value`] type.
//!
//! The main trait is [`InteropValue`]. It is implemented for all Rust types
//! which can be converted into and from a [`Value`] through the [`From`] and
//! [`TryFrom`] traits, respectively.
//!
//! Then, the [`InteropValueList`] trait is a layer on top, allowing the same
//! conversions but instead for tuples/lists containing multiple values.

#![no_std]
#![deny(
    clippy::missing_safety_doc,
    clippy::undocumented_unsafe_blocks,
    unsafe_op_in_unsafe_fn
)]

extern crate alloc;

use wasm::{
    addrs::FuncAddr,
    config::Config,
    value::{ExternAddr, Ref, ValueTypeMismatchError},
    FuncType, HaltExecutionError, NumType, RefType, ResultType, RuntimeError, Store, ValType,
    Value,
};

use alloc::{fmt::Debug, vec, vec::Vec};

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
    use alloc::vec::Vec;
    use wasm::addrs::{Addr, FuncAddr};
    use wasm::value::{ExternAddr, Value, ValueTypeMismatchError};

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
        let rust_value: RefFunc = RefFunc(Some(FuncAddr::new(0)));
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
        let rust_values: (f32, RefFunc) = (3.0, RefFunc(Some(FuncAddr::new(0))));
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

/// Helper function to quickly construct host functions without worrying about wasm to Rust
/// type conversion. For reading/writing user data into the current configuration, simply move
/// `user_data` into the passed closure.
/// # Example
/// ```
/// use wasm::{validate,  Store, host_function_wrapper, Value, HaltExecutionError};
/// fn my_wrapped_host_func(user_data: &mut (), params: Vec<Value>) -> Result<Vec<Value>, HaltExecutionError> {
///     host_function_wrapper(params, |(x, y): (u32, i32)| -> Result<u32, HaltExecutionError> {
///         let _user_data = user_data;
///         Ok(x + (y as u32))
///     })
/// }
/// fn main() {
///     let mut store = Store::new(());
///     // SAFETY: Parameters and result types do not contain address types.
///     let foo_bar = unsafe { store.func_alloc_typed_unchecked::<(u32, i32), u32>(my_wrapped_host_func) };
/// }
/// ```
pub fn host_function_wrapper<Params: InteropValueList, Results: InteropValueList>(
    params: Vec<Value>,
    f: impl FnOnce(Params) -> Result<Results, HaltExecutionError>,
) -> Result<Vec<Value>, HaltExecutionError> {
    let params =
        Params::try_from_values(params.into_iter()).expect("Params match the actual parameters");
    f(params).map(Results::into_values)
}

pub trait StoreTypedInvocationExt<T: Config> {
    /// Allocates a new function with a statically known type signature with some host code.
    ///
    /// This function is simply syntactic sugar for calling
    /// [`Store::func_alloc`] with statically know types.
    ///
    /// # Safety
    ///
    /// Same as [`Store::func_alloc_unchecked`].
    unsafe fn func_alloc_typed_unchecked<Params: InteropValueList, Returns: InteropValueList>(
        &mut self,
        host_func: fn(&mut T, Vec<Value>) -> Result<Vec<Value>, HaltExecutionError>,
    ) -> FuncAddr;

    /// Invokes a function with a statically known type signature without fuel.
    ///
    /// This function is simply syntactic sugar for calling [`Store::invoke`]
    /// without any fuel and destructuring the resulting
    /// [`RunState`](wasm::resumable::RunState) with statically known types.
    ///
    /// # Safety
    ///
    /// The caller has to guarantee that the given [`FuncAddr`] and any
    /// [`FuncAddr`] or [`ExternAddr`] values contained in the parameter values
    /// came from the current [`Store`] object.
    unsafe fn invoke_typed_without_fuel_unchecked<
        Params: InteropValueList,
        Returns: InteropValueList,
    >(
        &mut self,
        function: FuncAddr,
        params: Params,
    ) -> Result<Returns, RuntimeError>;
}

impl<T: Config> StoreTypedInvocationExt<T> for Store<'_, T> {
    unsafe fn func_alloc_typed_unchecked<Params: InteropValueList, Returns: InteropValueList>(
        &mut self,
        host_func: fn(&mut T, Vec<Value>) -> Result<Vec<Value>, HaltExecutionError>,
    ) -> FuncAddr {
        let func_type = FuncType {
            params: ResultType {
                valtypes: Vec::from(Params::TYS),
            },
            returns: ResultType {
                valtypes: Vec::from(Returns::TYS),
            },
        };
        // SAFETY: The caller makes the same safety guarantees that are required
        // by this function.
        unsafe { self.func_alloc_unchecked(func_type, host_func) }
    }

    unsafe fn invoke_typed_without_fuel_unchecked<
        Params: InteropValueList,
        Returns: InteropValueList,
    >(
        &mut self,
        function: FuncAddr,
        params: Params,
    ) -> Result<Returns, RuntimeError> {
        // SAFETY: The caller ensures that the given function address and all
        // address types contained in the parameters are valid in the current
        // store.
        let return_values =
            unsafe { self.invoke_without_fuel_unchecked(function, params.into_values()) }?;

        Returns::try_from_values(return_values.into_iter())
            .map_err(|ValueTypeMismatchError| RuntimeError::FunctionInvocationSignatureMismatch)
    }
}
