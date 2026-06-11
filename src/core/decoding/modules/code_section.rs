use alloc::vec::Vec;
use core::iter;

use crate::{
    core::{decoding::decoder::WasmDecoder, utils::ToUsizeExt},
    DecodingError, ValType,
};

pub fn decode_locals(wasm: &mut WasmDecoder) -> Result<Vec<ValType>, DecodingError> {
    let locals: Vec<(usize, ValType)> = wasm
        .decode_vec_map(|wasm| {
            let n = wasm.decode_var_u32()?.into_usize();
            let valtype = ValType::decode(wasm)?;

            Ok((n, valtype))
        })
        .and_then(Iterator::collect)?;

    // these checks are related to the official test suite binary.wast file, the first 2 assert_malformed's starting at line 350
    // we check to not have more than 2^32-1 locals, and if that number is okay, we then get to instantiate them all
    // this is because the flat_map and collect take an insane amount of time
    // in total, these 2 tests take more than 240s
    let mut total_no_of_locals: u64 = 0;
    for local in &locals {
        let temp = local.0 as u64;
        if temp > u32::MAX.into() {
            return Err(DecodingError::TooManyLocals(total_no_of_locals));
        };
        total_no_of_locals = match total_no_of_locals.checked_add(temp) {
            None => return Err(DecodingError::TooManyLocals(total_no_of_locals)),
            Some(n) => n,
        }
    }

    if total_no_of_locals > u32::MAX.into() {
        return Err(DecodingError::TooManyLocals(total_no_of_locals));
    }

    // Flatten local types for easier representation where n > 1
    let locals = locals
        .into_iter()
        .flat_map(|entry| iter::repeat_n(entry.1, entry.0))
        .collect::<Vec<ValType>>();

    Ok(locals)
}
