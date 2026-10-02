use core::iter;

use crate::{
    core::{decoding::decoder::WasmDecoder, utils::ToUsizeExt},
    DecodingError, ValType,
};

pub fn decode_locals<'a, 'wasm>(
    wasm: &'a mut WasmDecoder<'wasm>,
) -> Result<impl ExactSizeIterator<Item = ValType> + use<'a, 'wasm>, DecodingError> {
    // First pass to decode all locals and check if their total number exceeds 2^32-1
    let mut total_number_of_locals = 0_u32;
    let mut wasm_cloned = wasm.clone();

    wasm_cloned.decode_vec()?.try_for_each(|wasm| {
        let n = wasm.decode_var_u32()?;
        let _valtype = ValType::decode(wasm)?;

        total_number_of_locals = total_number_of_locals
            .checked_add(n)
            .ok_or(DecodingError::TooManyLocals)?;

        Ok(())
    })?;

    // Second pass to flatten locals
    let locals = wasm
        .decode_vec()
        .expect("locals were valid during first pass")
        .flat_map(|wasm| {
            let n = wasm.decode_var_u32().unwrap();
            let valtype = ValType::decode(wasm).unwrap();

            iter::repeat_n(valtype, n.into_usize())
        });

    Ok(WithExactSize {
        i: locals,
        len: total_number_of_locals.into_usize(),
    })
}

struct WithExactSize<I> {
    i: I,
    len: usize,
}

impl<T, I: Iterator<Item = T>> Iterator for WithExactSize<I> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.i.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<T, I: Iterator<Item = T>> ExactSizeIterator for WithExactSize<I> {
    fn len(&self) -> usize {
        self.len
    }
}
