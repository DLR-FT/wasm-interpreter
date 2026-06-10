use alloc::{vec, vec::Vec};

use crate::{core::utils::ToUsizeExt, Limits, Ref, RuntimeError, TableType, TrapError};

#[derive(Debug)]
pub struct TableInst {
    pub ty: TableType,
    pub elem: Vec<Ref>,
}

impl TableInst {
    pub fn len(&self) -> usize {
        self.elem.len()
    }

    /// <https://webassembly.github.io/spec/core/exec/modules.html#growing-tables>
    pub fn grow(&mut self, n: u32, reff: Ref) -> Result<(), RuntimeError> {
        // TODO refactor error, the spec Table.grow raises Table.{SizeOverflow, SizeLimit, OutOfMemory}
        let len = n
            .checked_add(self.elem.len() as u32)
            .ok_or(TrapError::TableOrElementAccessOutOfBounds)?;

        // roughly matches step 4,5,6
        // checks limits_prime.valid() for limits_prime := { min: len, max: self.ty.lim.max }
        // https://webassembly.github.io/spec/core/valid/types.html#limits
        if self.ty.lim.max.map(|max| len > max).unwrap_or(false) {
            return Err(TrapError::TableOrElementAccessOutOfBounds.into());
        }
        let limits_prime = Limits {
            min: len,
            max: self.ty.lim.max,
        };

        self.elem.extend(vec![reff; n.into_usize()]);

        self.ty.lim = limits_prime;
        Ok(())
    }
}
