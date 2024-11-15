//! This module contains a data structure to allow in-place interpretation
//!
//! Control-flow in WASM is denoted in labels. To avoid linear search through the WASM binary or
//! stack for the respective label of a branch, a sidetable is generated during validation, which
//! stores the offset on the current instruction pointer for the branch. A sidetable entry hence
//! allows to translate the implicit control flow information ("jump to the next `else`") to
//! explicit modifications of the instruction pointer (`instruction_pointer += 13`).
//!
//! Branches in WASM can only go outwards, they either `break` out of a block or `continue` to the
//! head of a loob block. Put differently, a label can only be referenced from within its
//! associated structured control instruction.
//!
//! Noteworthy, branching can also have side-effects on the operand stack:
//!
//! - Taking a branch unwinds the operand stack, down to where the targeted structured control flow
//!   instruction was entered. [`SidetableEntry::popcnt`] holds information on how many values to
//!   pop from the operand stack when a branch is taken.
//! - When a branch is taken, it may consume arguments from the operand stack. These are pushed
//!   back on the operand stack after unwinding. This behavior can be emulated by copying the
//!   uppermost [`SidetableEntry::valcnt`] operands on the operand stack before taking a branch
//!   into a structured control instruction.
//!   
//! # Relevant instructions
//! **Sidetable jump origins (and how many ST entries they require)**
//! - br (1)
//! - br_if (1)
//! - br_table (num_labels + 1 for default label)
//! - if (2, maybe 1??)

//! **Sidetable jump targets**
//! - end of block
//! - loop
//! - else
//! - end of else block
//!
//! # Reference
//!
//! - Core / Syntax / Instructions / Control Instructions, WASM Spec,
//!   <https://webassembly.github.io/spec/core/syntax/instructions.html#control-instructions>
//! - "A fast in-place interpreter for WebAssembly", Ben L. Titzer,
//!   <https://arxiv.org/abs/2205.01183>

use alloc::vec::Vec;

use crate::{Error, Result};

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
#[derive(Copy, Clone)]
pub struct SidetableEntry {
    /// Δpc: the amount to adjust the instruction pointer by if the branch is taken
    pub delta_pc: isize,

    /// Δstp: the amount to adjust the side-table index by if the branch is taken
    pub delta_stp: isize,

    /// valcnt: the number of values that will be copied if the branch is taken
    ///
    /// Branches may additionally consume operands themselves, which they push back on the operand
    /// stack after unwinding.
    pub val_count: usize,

    /// popcnt: the number of values that will be popped if the branch is taken
    ///
    /// Taking a branch unwinds the operand stack down to the height where the targeted structured
    /// control instruction was entered.
    pub pop_count: usize,
}

impl core::fmt::Debug for SidetableEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SidetableEntry")
            .field("Δpc", &self.delta_pc)
            .field("Δstp", &self.delta_stp)
            .field("valcnt", &self.val_count)
            .field("popcnt", &self.pop_count)
            .finish()
    }
}

pub struct IncompleteSidetableEntry {
    pub ip: usize,
    pub delta_ip: Option<isize>,
    pub delta_stp: Option<isize>,
    pub val_count: usize,
    pub pop_count: usize,
}

pub struct SidetableBuilder(pub Vec<IncompleteSidetableEntry>);

impl SidetableBuilder {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    // This panics if some sidetable entries still contain any None fields.
    pub fn into_sidetable(self) -> Sidetable {
        self.0
            .into_iter()
            .map(|entry| SidetableEntry {
                delta_pc: entry.delta_ip.expect("Failed to generate sidetable"),
                delta_stp: entry.delta_stp.expect("Failed to generate sidetable"),
                val_count: entry.val_count,
                pop_count: entry.pop_count,
            })
            .collect()
    }
}
