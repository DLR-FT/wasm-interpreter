use core::{convert::Infallible, iter};

use crate::{
    core::{
        decoding::decoder::WasmDecoder,
        trusted_exact_size_iterator::{IntoTrustedUncheckedExt, TrustedExactSizeIterator},
        utils::ToUsizeExt,
    },
    DecodingError, ValType,
};

pub fn decode_locals<'a, 'wasm>(
    wasm: &'a mut WasmDecoder<'wasm>,
) -> Result<impl TrustedExactSizeIterator<Item = ValType> + use<'a, 'wasm>, DecodingError> {
    // First pass to decode all locals and check if their total number exceeds 2^32-1
    let mut total_number_of_locals = 0_u32;
    let mut wasm_cloned = wasm.clone();

    wasm_cloned
        .decode_vec(|wasm, _| {
            let n = wasm.decode_var_u32()?;
            let _valtype = ValType::decode(wasm)?;

            total_number_of_locals = total_number_of_locals
                .checked_add(n)
                .ok_or(DecodingError::TooManyLocals)?;

            Ok(())
        })?
        .collect::<Result<(), _>>()?;

    // Second pass to flatten locals
    let locals_untrusted = wasm
        .decode_vec(|wasm, _| {
            let n = wasm.decode_var_u32().unwrap();
            let valtype = ValType::decode(wasm).unwrap();
            Ok::<_, Infallible>((n, valtype))
        })
        .expect("locals were valid during first pass")
        .flat_map(|Ok((n, valtype))| iter::repeat_n(valtype, n.into_usize()));

    let locals =
        unsafe { locals_untrusted.into_trusted_unchecked(total_number_of_locals.into_usize()) };

    Ok(locals)
}
