use crate::{
    core::{fixed_capacity_vec::FixedCapacityVec, utils::ToUsizeExt},
    Config, Limits, Ref, RuntimeError, TableType,
};

#[derive(Debug)]
pub struct TableInst {
    pub ty: TableType,
    pub elem: FixedCapacityVec<Ref>,
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

        // If the capacity does not suffice, call the user to ask for a reallocation.
        if let Some(additional_capacity) = len.into_usize().checked_sub(self.elem.capacity()) {
            let num_new_elements = T::table_requested_allocation(
                Some(self.elem.capacity()),
                additional_capacity,
                limits.max.map(u32::into_usize),
            )
            .ok_or(RuntimeError::HostRefusedAllocation)?;

            // If the user chose a reallocation size lower than the required additional capacity, it
            // is okay to return early and perform no realloc at all.
            if num_new_elements < additional_capacity {
                return Err(RuntimeError::TableGrowOverflowed);
            }

            if self.elem.capacity().checked_add(num_new_elements).is_none() {
                return Err(RuntimeError::TableGrowOverflowed);
            }

            unsafe { self.elem.extend_reserve_unchecked(num_new_elements) };
        }

        // 7. Append ref^N to tableinst.elem.
        debug_assert!(self
            .elem
            .len()
            .checked_add(n.into_usize())
            .is_some_and(|new_len| new_len <= self.elem.capacity()));
        // SAFETY: The capacity check ensures that enough capacity is available for at least n more
        // elements.
        unsafe { self.elem.push_n_unchecked(reff, n.into_usize()) };

        // 8. Set tableinst.type to the table type limits' t.
        self.ty.lim = limits_prime;

        Ok(())
    }
}
