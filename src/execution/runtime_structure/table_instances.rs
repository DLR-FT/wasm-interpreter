use core::iter;

use alloc::vec::Vec;

use crate::{core::utils::ToUsizeExt, Config, Limits, Ref, RuntimeError, TableType};

#[derive(Debug)]
pub struct TableInst {
    pub ty: TableType,
    pub elem: Vec<Ref>,
}

impl TableInst {
    pub fn len(&self) -> u32 {
        self.elem
            .len()
            .try_into()
            .expect("table length can not be larger than or equal to 2^32")
    }

    /// See: [WebAssembly Specification 2.0 - 4.5.3.8 Growing Tables](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#growing-tables%E2%91%A0).
    pub fn grow<T: Config>(&mut self, n: u32, reff: Ref) -> Result<(), RuntimeError> {
        // 1. Let tableinst be the table instance to grow, n the number of elements by which to grow
        //    it, and ref the initialization value.
        //
        // self is the table instance, n the number of elements by which to grow it and rref the
        // initialization value.

        // 2. Let len be n added to the length of tableinst.elem.
        let len = n.checked_add(self.len());

        // 3. If len is larger than or equal to 2^32, then fail.
        let len = len.ok_or(RuntimeError::TableGrowOverflowed)?;

        // 4. Let limits t be the structure of table type tableinst.type.
        let limits = self.ty.lim;

        // 5. Let limits' be limits with min updated to len.
        let limits_prime = Limits { min: len, ..limits };

        // 6. If limits' is not valid, then fail.
        if limits_prime.max.is_some_and(|max| limits_prime.min > max) {
            return Err(RuntimeError::TableGrowExceededLimit);
        }

        if let Some(max_elements) = T::MAX_NUMBER_OF_TABLE_ELEMENTS {
            if len.into_usize() > max_elements.get() {
                return Err(RuntimeError::TableGrowOverflowed);
            }
        }

        // 7. Append ref^N to tableinst.elem.
        self.elem.extend(iter::repeat_n(reff, n.into_usize()));

        // 8. Set tableinst.type to the table type limits' t.
        self.ty.lim = limits_prime;

        Ok(())
    }
}
