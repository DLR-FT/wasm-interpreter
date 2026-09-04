use alloc::{boxed::Box, fmt};

use crate::core::{decoding::decoder::span::Span, structure::modules::indices::MemIdx};

#[derive(Clone)]
pub struct DataSegment {
    pub init: Box<[u8]>,
    pub mode: DataMode,
}

impl fmt::Debug for DataSegment {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DataSegment")
            .field("mode", &self.mode)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Debug)]
pub enum DataMode {
    Passive,
    Active(DataModeActive),
}

#[derive(Clone, Debug)]
pub struct DataModeActive {
    pub memory_idx: MemIdx,
    pub offset: Span,
}
