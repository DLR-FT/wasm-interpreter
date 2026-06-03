//! All `&'static` and `&'static mut`s handed out by functions in this crate are heap allocated and
//! **have** to be freed again to avoid memory leakage.
#![no_std]

extern crate alloc;

mod cursed;

/// Type for user data, just a void pointer, to allow arbitrary context to be associated with the
/// interpreter.
#[repr(transparent)]
pub struct UserDataType(*mut core::ffi::c_void);

impl wasm::config::Config for UserDataType {
    const MAX_VALUE_STACK_SIZE: usize = 0xf0000;
    const MAX_CALL_STACK_SIZE: usize = 0x1000;
}

/// Result of an operation; either success (`Ok`) or an error
#[repr(C)]
pub enum CResult<T> {
    Ok(T),
    Err(CError),
}

/// Error cause for a failed operation
#[repr(C)]
pub enum CError {
    /// Validation failed
    ValidationError,

    /// The validation info is invalid, i.e. a nullptr
    InvalidValidationInfo,

    /// Instantiation failed
    InstantiationError,

    /// The store is invalid, i.e. a nullptr
    InvalidStore,

    // A string failed to parse as valid UTF-8
    UTF8Error,

    /// A C-str is not null terminated
    CStrNotNullTerminated,

    NoSuchExport,

    ExternAddrIsNotAFunc,

    RuntimeError,
}

/// Opaque wrapper for a Wasm store. Stores the state of an interpreter instance.
pub struct Store(pub(crate) wasm::Store<'static, UserDataType>);

/// Initialize a new store. See
/// <https://webassembly.github.io/spec/core/appendix/embedding.html#mathrm-store-init-xref-exec-runtime-syntax-store-mathit-store> for background.
///
/// # Safety
///
/// - Ensure that `new_store` is a valid pointer to an opaque pointer.
/// - Ensure that `new_store` is only ever used if the return value was `CResult::Ok`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn store_init(user_data: UserDataType) -> CResult<&'static mut Store> {
    CResult::Ok(cursed::alloc(Store(wasm::Store::new(user_data))))
}

/// Drop a store.
///
/// This deallocates a [`Store`] on the heap.
///
/// # Safety
///
/// - Ensure that `store` is a valid pointer to an initialized store.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn store_drop(store: &'static mut Store) -> CResult<()> {
    cursed::dealloc(store);
    CResult::Ok(())
}

/// Opaque wrapper for a Wasm module. Stores validation information and a reference to the
/// corresponding bytes of the bytecode making up a Wasm module.
pub struct Module {
    pub(crate) validation_info: wasm::ValidationInfo<'static>,
}

/// Decode/parse and validate Wasm bytecode.
///
/// # Safety
///
/// - Ensure that `wasm_bytecode` is a pointer living at least until when the resulting module is
///   freed.
/// - Ensure that `new_module` is a valid pointer to an opaque pointer.
/// - Ensure that `new_store` is only ever used if the return value was `CResult::Ok`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn module_validate(
    wasm_bytecode: *const u8,
    wasm_bytecode_length: usize,
) -> CResult<&'static mut Module> {
    // SAFETY: Assumes that `wasm_bytecode` points to data living long enough until the module
    // is deallocated.
    let wasm_slice: &'static [_] =
        unsafe { core::slice::from_raw_parts(wasm_bytecode, wasm_bytecode_length) };

    match wasm::validate(wasm_slice) {
        Ok(validation_info) => CResult::Ok(cursed::alloc(Module { validation_info })),
        Err(_) => CResult::Err(CError::ValidationError),
    }
}

/// Drop a module.
///
/// This deallocates a [`Module`] on the heap.
///
/// # Safety
///
/// - Ensure that `module` is a valid pointer to an initialized module.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn module_drop(module: &mut Module) -> CResult<()> {
    cursed::dealloc(module);
    CResult::Ok(())
}

/// Opaque wrapper for a Wasm module instantion outcome. Stores the module instance's address and
/// its exports.
pub struct ModuleInst {
    pub(crate) module_addr: wasm::addrs::ModuleAddr,
    pub(crate) module_exports: alloc::vec::Vec<(alloc::string::String, wasm::ExternVal)>,
}

/// Instantiate a module into a store.
///
/// # Safety
///
/// - Ensure that `store` is a valid pointer to an intialized store.
/// - Ensure that `module` is a valid pointer to a validated module.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn module_instantiate(
    store: &mut Store,
    module: &mut Module,
) -> CResult<&'static mut ModuleInst> {
    match unsafe {
        store
            .0
            .module_instantiate(&module.validation_info, alloc::vec::Vec::new(), None)
    } {
        Ok(instantiation_outcome) => {
            // SAFETY: the `ModuleAddr` just came from instantiation that module.
            let module_exports =
                unsafe { store.0.instance_exports(instantiation_outcome.module_addr) };

            CResult::Ok(cursed::alloc(ModuleInst {
                module_addr: instantiation_outcome.module_addr,
                module_exports,
            }))
        }
        Err(_) => CResult::Err(CError::InstantiationError),
    }
}

/// Drop a module instance.
///
/// This deallocates a [`ModuleInstance`] on the heap.
///
/// # Safety
///
/// - Ensure that `module_inst` is a valid pointer to an initialized module.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn module_inst_drop(module_inst: &mut ModuleInst) -> CResult<()> {
    cursed::dealloc(module_inst);
    CResult::Ok(())
}

/// Opaque wrapper for the external address of a Wasm module's export. Use this to get a handle i.e.
/// to an function in the Wasm module.
pub struct ExternAddr {
    addr: wasm::ExternVal,
}

/// # Safety
///
/// Ensure that name is a pointer to a c_char array being, that is null terminated and at least
/// `name_length` chars long.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn instance_export(
    module_inst: &ModuleInst,
    name: *const core::ffi::c_uchar,
    name_length: usize,
) -> CResult<&'static mut ExternAddr> {
    // SAFETY: assumes that `name` is a pointer to a C string at least `name_length` c_chars long
    // (including the null byte into the count).
    let name_c_char_slice = unsafe { core::slice::from_raw_parts(name, name_length) };

    let Ok(name_c_str) = core::ffi::CStr::from_bytes_until_nul(name_c_char_slice) else {
        return CResult::Err(CError::CStrNotNullTerminated);
    };
    let Ok(name_str) = name_c_str.to_str() else {
        return CResult::Err(CError::UTF8Error);
    };

    module_inst
        .module_exports
        .iter()
        .find(|(candidate_name, _)| candidate_name == name_str)
        .map(|(_, addr)| cursed::alloc(ExternAddr { addr: *addr }))
        .map(CResult::Ok)
        .unwrap_or(CResult::Err(CError::NoSuchExport))
}

// TODO
// - arguments
// - return values
#[unsafe(no_mangle)]
pub unsafe extern "C" fn func_invoke(
    store: &mut Store,
    extern_addr: &ExternAddr,
    fuel: u64,
) -> CResult<()> {
    let maybe_fuel = match fuel {
        0 => None,
        n => Some(n),
    };

    let Some(func) = extern_addr.addr.as_func() else {
        return CResult::Err(CError::ExternAddrIsNotAFunc);
    };

    let maybe_resumable = unsafe { store.0.create_resumable(func, alloc::vec![], maybe_fuel) };
    let Ok(resumable) = maybe_resumable else {
        return CResult::Err(CError::RuntimeError);
    };

    let Some(resumable) = resumable.as_wasm() else {
        return CResult::Err(CError::RuntimeError);
    };

    // SAFETY: Only one store is used. Therefore, this must always be
    // the correct one.
    unsafe { store.0.resume_wasm(resumable) }.unwrap();

    CResult::Ok(())
}
