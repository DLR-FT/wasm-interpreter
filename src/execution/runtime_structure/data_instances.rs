use alloc::vec::Vec;

pub struct DataInst {
    pub data: Vec<u8>,
}

impl core::fmt::Debug for DataInst {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DataInst").finish_non_exhaustive()
    }
}
