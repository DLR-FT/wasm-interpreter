//! TODO: Logic that makes sure that objects, which belong to a specific [`Store`], are only used with that [`Store`].

use core::{
    num::NonZeroU32,
    sync::atomic::{AtomicU64, Ordering},
};

use alloc::vec::Vec;

use crate::{
    config::Config,
    core::reader::types::FuncType,
    resumable::{ResumableRef, RunState},
    value::{ExternAddr, Ref, F32, F64},
    RefType, RuntimeError, ValidationInfo, Value,
};

use super::{
    addrs::{FuncAddr, GlobalAddr, MemAddr, ModuleAddr, TableAddr},
    ExternVal, FuncInst, HaltExecutionError, Store, StoredHostFuncInst,
};

/// A unique identifier for a specfic [`Store]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct StoreId(u64);

impl StoreId {
    /// Creates a new unique [`StoreId`]
    #[allow(clippy::new_without_default)] // reason = "StoreId::default() might be misunderstood to be some default value. However, a
                                          // default value does not exist in that sense because every newly created StoreId must be unique"
    pub fn new() -> Self {
        static NEXT_STORE_ID: AtomicU64 = AtomicU64::new(0);

        // TODO find a fix for the default wrapping behaviour of `fetch_add`.
        // Maybe we could return a `RuntimeError` here?
        Self(NEXT_STORE_ID.fetch_add(1, Ordering::SeqCst))
    }
}

pub struct CheckedStore<'b, T: Config> {
    inner: Store<'b, T>,
    /// A unique identifier for this store. This is used to check if a foreign `T`, thats wrapped
    /// in a [`Stored<...>`] object belongs to this store. See the [`stored`] module for more info.
    id: StoreId,
}

#[derive(Copy, Clone, Debug)]
pub struct Stored<T> {
    id: StoreId,
    inner: T,
}

pub enum StoredValue {
    I32(u32),
    I64(u64),
    F32(F32),
    F64(F64),
    V128([u8; 16]),
    Ref(StoredRef),
}

#[derive(Clone, Copy, Debug)]
pub enum StoredRef {
    Null(RefType),
    Func(Stored<FuncAddr>),
    Extern(Stored<ExternAddr>),
}

/// The stored variant of [`ExternVal`]
#[derive(Debug, Copy, Clone)]
pub enum StoredExternVal {
    Func(Stored<FuncAddr>),
    Table(Stored<TableAddr>),
    Mem(Stored<MemAddr>),
    Global(Stored<GlobalAddr>),
}

/// The stored variant of [`RunState`]
pub enum StoredRunState {
    Finished(Vec<StoredValue>),
    Resumable {
        resumable_ref: Stored<ResumableRef>,
        required_fuel: NonZeroU32,
    },
}

impl<T: Config> CheckedStore<'_, T> {
    pub(crate) fn wrap_stored<U>(&self, inner: U) -> Stored<U> {
        Stored { id: self.id, inner }
    }

    pub(crate) fn try_unwrap_stored<U>(&self, stored: Stored<U>) -> Result<U, RuntimeError> {
        if self.id == stored.id {
            Ok(stored.inner)
        } else {
            Err(RuntimeError::StoreIdMismatch)
        }
    }

    pub(crate) fn wrap_value(&self, value: Value) -> StoredValue {
        match value {
            Value::I32(x) => StoredValue::I32(x),
            Value::I64(x) => StoredValue::I64(x),
            Value::F32(x) => StoredValue::F32(x),
            Value::F64(x) => StoredValue::F64(x),
            Value::V128(x) => StoredValue::V128(x),

            Value::Ref(Ref::Null(ty)) => StoredValue::Ref(StoredRef::Null(ty)),
            Value::Ref(Ref::Func(func_addr)) => {
                StoredValue::Ref(StoredRef::Func(self.wrap_stored(func_addr)))
            }
            Value::Ref(Ref::Extern(extern_addr)) => {
                StoredValue::Ref(StoredRef::Extern(self.wrap_stored(extern_addr)))
            }
        }
    }

    pub(crate) fn try_unwrap_value(
        &self,
        stored_value: StoredValue,
    ) -> Result<Value, RuntimeError> {
        let value = match stored_value {
            StoredValue::I32(x) => Value::I32(x),
            StoredValue::I64(x) => Value::I64(x),
            StoredValue::F32(x) => Value::F32(x),
            StoredValue::F64(x) => Value::F64(x),
            StoredValue::V128(x) => Value::V128(x),
            StoredValue::Ref(StoredRef::Null(ty)) => Value::Ref(Ref::Null(ty)),
            StoredValue::Ref(StoredRef::Func(func_addr)) => {
                Value::Ref(Ref::Func(self.try_unwrap_stored(func_addr)?))
            }
            StoredValue::Ref(StoredRef::Extern(extern_addr)) => {
                Value::Ref(Ref::Extern(self.try_unwrap_stored(extern_addr)?))
            }
        };

        Ok(value)
    }

    pub(crate) fn wrap_extern_val(&self, extern_val: ExternVal) -> StoredExternVal {
        match extern_val {
            ExternVal::Func(addr) => StoredExternVal::Func(self.wrap_stored(addr)),
            ExternVal::Table(addr) => StoredExternVal::Table(self.wrap_stored(addr)),
            ExternVal::Mem(addr) => StoredExternVal::Mem(self.wrap_stored(addr)),
            ExternVal::Global(addr) => StoredExternVal::Global(self.wrap_stored(addr)),
        }
    }

    pub(crate) fn wrap_run_state(&self, run_state: RunState) -> StoredRunState {
        match run_state {
            RunState::Finished(values) => StoredRunState::Finished(
                values
                    .into_iter()
                    .map(|value| self.wrap_value(value))
                    .collect(),
            ),
            RunState::Resumable {
                resumable_ref,
                required_fuel,
            } => StoredRunState::Resumable {
                resumable_ref: self.wrap_stored(resumable_ref),
                required_fuel,
            },
        }
    }
}

impl<'b, T: Config> CheckedStore<'b, T> {
    pub fn new(user_data: T) -> Self {
        Self {
            inner: Store::new(user_data),
            id: StoreId::new(),
        }
    }

    /// Instantiates a validated module with `validation_info` as validation evidence with name `name`.
    ///
    /// This is the safe variant of [`Store::add_module_unchecked`]. It uses
    /// _stored_ variants of arguments/return types to be able to track which
    /// [`Store`] instance they belong to. This is important, because internally
    /// the interpreter assumes that all references into the [`Store`] (e.g.
    /// address types) are always valid.
    pub fn add_module(
        &mut self,
        name: &str,
        validation_info: &ValidationInfo<'b>,
        maybe_fuel: Option<u32>,
    ) -> Result<Stored<ModuleAddr>, RuntimeError> {
        self.inner
            .add_module_unchecked(name, validation_info, maybe_fuel)
            .map(|module_addr| self.wrap_stored(module_addr))
    }

    /// Gets an export of a specific module instance by its name
    ///
    /// This is the safe variant of [`Store::instance_export_unchecked`]. It uses
    /// _stored_ variants of arguments/return types to be able to track which
    /// [`Store`] instance they belong to. This is important, because internally
    /// the interpreter assumes that all references into the [`Store`] (e.g.
    /// address types) are always valid.
    pub fn instance_export(
        &self,
        module_addr: Stored<ModuleAddr>,
        name: &str,
    ) -> Result<StoredExternVal, RuntimeError> {
        let module_addr = self.try_unwrap_stored(module_addr)?;
        self.inner
            .instance_export_unchecked(module_addr, name)
            .map(|extern_val| self.wrap_extern_val(extern_val))
    }

    /// Allocates a new function with some host code.
    ///
    /// This is the safe variant of [`Store::func_alloc_unchecked`]. It uses
    /// _stored_ variants of arguments/return types to be able to track which
    /// [`Store`] instance they belong to. This is important, because internally
    /// the interpreter assumes that all references into the [`Store`] (e.g.
    /// address types) are always valid.
    // TODO unfortunatly safe host functions can not be implemented as a layer
    // around the pre-existing `Store::func_alloc_unchecked` solution. This is
    // because we use fn pointers for host functions.
    pub fn func_alloc(
        &mut self,
        func_type: FuncType,
        host_func: fn(&mut T, Vec<StoredValue>) -> Result<Vec<StoredValue>, HaltExecutionError>,
    ) -> Stored<FuncAddr> {
        // 1. Pre-condition: `functype` is valid.

        // 2. Let `funcaddr` be the result of allocating a host function in `store` with
        //    function type `functype` and host function code `hostfunc`.
        // 3. Return the new store paired with `funcaddr`.
        //
        // Note: Returning the new store is a noop for us because we mutate the store instead.
        let func_addr = self
            .functions
            .insert(FuncInst::StoredHostFunc(StoredHostFuncInst {
                function_type: func_type,
                hostcode: host_func,
            }));

        self.wrap_stored(func_addr)
    }

    /// Gets the type of a function by its addr.
    ///
    /// This is the safe variant of [`Store::func_type_unchecked`]. It uses
    /// _stored_ variants of arguments/return types to be able to track which
    /// [`Store`] instance they belong to. This is important, because internally
    /// the interpreter assumes that all references into the [`Store`] (e.g.
    /// address types) are always valid.
    pub fn func_type(&self, func_addr: Stored<FuncAddr>) -> Result<FuncType, RuntimeError> {
        let func_addr = self.try_unwrap_stored(func_addr)?;
        Ok(self.func_type_unchecked(func_addr))
    }

    /// This is the safe variant of [`Store::invoke_unchecked`]. It uses
    /// _stored_ variants of arguments/return types to be able to track which
    /// [`Store`] instance they belong to. This is important, because internally
    /// the interpreter assumes that all references into the [`Store`] (e.g.
    /// address types) are always valid.
    pub fn invoke(
        &mut self,
        func_addr: Stored<FuncAddr>,
        params: Vec<StoredValue>,
        maybe_fuel: Option<u32>,
    ) -> Result<StoredRunState, RuntimeError> {
        let func_addr = self.try_unwrap_stored(func_addr)?;
        let params = params
            .into_iter()
            .map(|value| self.try_unwrap_value(value))
            .collect::<Result<Vec<Value>, RuntimeError>>()?;

        self.invoke_unchecked(func_addr, params, maybe_fuel)
            .map(|run_state| self.wrap_run_state(run_state))
    }
}
