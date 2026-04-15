//! Trace probes for coverage measurement
//!
//! Contains different probes to record execution traces

// TODO build abstraction to represent a trace, maybe via a double ended iterator?
// pub struct Trace {
//     // Internal representation of a
//     trace: alloc::vec::Vec<usize>,
// }

#[cfg(feature = "alloc")]
use crate::covlist::CovList;

/// Trace every instruction which is executed, recording to a [`Vec`]
///
/// This is a rather naive solution, it would suffice to only trace the basic blocks. This is mainly
/// useful to verify other trace approaches, as it provides an unrefutable truthy trace of what was
/// executed, in which order.
#[cfg(feature = "alloc")]
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct FullTraceToVec {
    pub trace: alloc::vec::Vec<u64>,
}

#[cfg(feature = "alloc")]
impl wasm::config::Config for FullTraceToVec {
    fn instruction_hook(&mut self, bytecode: &[u8], pc: usize) {
        self.trace.push(pc.try_into().unwrap());
        trace!("pc = {pc:#x?}, instruction = {:#02x?}", bytecode[pc]);
    }
}

/// Trace every basic block which is executed, recording to a [`CovList`]
///
/// This hook records the current program counter when a basic block is entered to and exited from,
/// recording the range into a `CovList`. Overlapping ranges are thus eliminated,
///
/// # Notes
///
/// - Since only the instruction ranges of the blocks are recorded, it is not possible to iterate over
/// instructions without re-parsing the bytecode, since instruction sizes differ.
#[cfg(feature = "alloc")]
#[derive(Debug, Default)]
pub struct BasicBlockTraceToCovList {
    /// ordered sequence of instruction ranges that were visited
    pub ranges: CovList,

    /// has the value true iff the next instruction is the first instruction of a basic block
    not_jumped_into: bool,

    /// the pc value when the first instruction of the most recently visited basic block is hit
    start_of_bb_instr: u64,
}

#[cfg(feature = "alloc")]
impl wasm::config::Config for BasicBlockTraceToCovList {
    fn instruction_hook(&mut self, bytecode: &[u8], pc: usize) {
        if !self.not_jumped_into {
            self.start_of_bb_instr = pc.try_into().unwrap();
            let instr = bytecode[pc];
            trace!("entering basic block with pc = {pc:#x?}, instruction = {instr:#02x?}");
            self.not_jumped_into = true;
        } else {
            use wasm::opcode::*;
            if let instr @ (UNREACHABLE | LOOP | IF | ELSE | END | BR | BR_IF | BR_TABLE | RETURN
            | CALL | CALL_INDIRECT) = bytecode[pc]
            {
                trace!("leaving basic block with pc = {pc:#x?}, instruction = {instr:#02x?}");
                self.ranges
                    .insert(self.start_of_bb_instr..(pc + 1).try_into().unwrap());
            }
        }
    }
}
