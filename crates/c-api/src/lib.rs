#![no_std]

use core::mem::MaybeUninit;

pub enum Module<'a> {
    BytesOnly(&'a [u8]),
    ValidationInfo(wasm::ValidationInfo<'a>),
}

#[repr(C)]
pub enum MaybeError {
    NoError = 0,
}

// #[no_mangle]
pub extern "C" fn module_parse<'a>(
    wasm_bytecode_ptr: *const u8,
    wasm_bytecode_size: usize,
    module: &mut MaybeUninit<Module<'a>>,
) -> MaybeError {
    todo!();
}

/// # Safety
///
/// TBD
#[unsafe(no_mangle)]
pub unsafe extern "C" fn module_validate(wasm_bytecode_ptr: *const u8, wasm_bytecode_size: usize) {
    let wasm_bytecode =
        unsafe { core::slice::from_raw_parts(wasm_bytecode_ptr, wasm_bytecode_size) };
    wasm::validate(wasm_bytecode).unwrap();
}
