//! This module contains a data structure to allow in-place interpretation
//!
//! Control-flow in WASM is denoted in labels. To avoid linear search through the WASM binary for
//! the respective label, we generate a sidetable that stores the offset on the current instruction
//! pointer for each branch. A sidetable entry hence allows to translate the implicit control flow
//! information ("jump to the next "else") to explicit modifications of the instruction pointer
//! (`instruction_pointer += 13`).
//!
//!
//! # Reference
//!
//! "A fast in-place interpreter for WebAssembly", Ben L. Titzer, https://arxiv.org/abs/2205.01183

use alloc::vec::Vec;

/// A sidetable
pub type Sidetable = Vec<SidetableEntry>;

/// Entry to translate the current branches implicit target into an explicit offset to the instruction pointer, as well as the side table pointer
///
/// Each of the following constructs requires a [`SidetableEntry`]:
///
/// - br
/// - br_if
/// - br_table
/// - else
pub struct SidetableEntry {
    /// Δip: the amount to adjust the instruction pointer by if the branch is taken
    delta_pc: isize,

    /// Δstp: the amount to adjust the side-table pointer by if the branch is taken
    delta_stp: isize,

    /// valcnt: the number of values that will be copied if the branch is taken
    valcnt: usize,

    /// popcnt: the number of values that will be popped if the branch is taken
    popcnt: usize,
}
