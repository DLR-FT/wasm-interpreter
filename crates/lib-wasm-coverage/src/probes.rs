//! Trace probes for coverage measurement
//!
//! Contains different probes to record execution traces
pub trait ExecutionTrace
where
    for<'a> &'a Self: IntoIterator<Item = u64>,
{
    fn covers(&self, instr_addr: u64) -> bool {
        self.into_iter().any(|instr| instr == instr_addr)
    }
}
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

pub struct FullTraceToVecIterator<'a>(core::iter::Copied<core::slice::Iter<'a, u64>>);

impl<'a> Iterator for FullTraceToVecIterator<'a> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'a> IntoIterator for &'a FullTraceToVec {
    type Item = u64;

    type IntoIter = FullTraceToVecIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        FullTraceToVecIterator(self.trace.iter().copied())
    }
}

impl ExecutionTrace for FullTraceToVec {}

/// Trace every basic block which is executed, recording to a [`Vec`]
///
/// Whenever a basic block finishes, this hook records the current program counter. Subsequently on
/// the first instruction of the next basic block it again records the program counter. Thus for
/// each basic block a record of its entry and exit instruction is recorded. This is redundant
/// information, however, it may be useful to validate the correctness of the algorithm to have both
/// entry and exit of the basic block.
///
/// # Notes
///
/// - No sane partitioning into basic blocks is conducted, i.e. a nested if may record as multiple
///   empty basic blocks one after another
/// - For the very first basic block in the execution trace, depending on how the interpreter loop
///   is entered, the start of the basic block may not be recorded (but the exit will be)
/// - For the very last basic block in the execution trace, depending on how the execution ends
///   (i.e. via a TRAP), the end of the basic block may not be recorded (but the start will be)
#[cfg(feature = "alloc")]
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct BasicBlockTraceToVec {
    /// Sequence of program counters
    pub trace: alloc::vec::Vec<u64>,

    /// Set to `true` before executing a basic block demarcating instruction, set to `false` just
    /// before executing the first instruction of the next basic block
    start_of_bb: bool,
}

#[cfg(feature = "alloc")]
impl wasm::config::Config for BasicBlockTraceToVec {
    fn instruction_hook(&mut self, bytecode: &[u8], pc: usize) {
        use wasm::opcode::*;
        match (bytecode[pc], self.start_of_bb) {
            (
                // TODO this mixes instructions that start a basic block with instructions that end
                // it. We only need one of the too. However, until further validation is conducted,
                // this is the safe bet.

                // END needs to be here too, as it can return from a function, which itself is a basic block
                // BLOCK is not a demarcator, as it is never targeted by a branch
                instr @ (UNREACHABLE | LOOP | IF | ELSE | END | BR | BR_IF | BR_TABLE | RETURN
                | CALL | CALL_INDIRECT),
                _,
            ) => {
                self.start_of_bb = true;
                self.trace.push(pc.try_into().unwrap());
                trace!("leaving basic block with pc = {pc:#x?}, instruction = {instr:#02x?}");
            }
            (instr, true) => {
                self.start_of_bb = false;
                self.trace.push(pc.try_into().unwrap());
                trace!("entering basic block with pc = {pc:#x?}, instruction = {instr:#02x?}");
            }
            _ => {}
        }
        if let IF | ELSE | END = bytecode[pc] {}
    }
}

#[cfg(feature = "alloc")]
#[derive(Debug, Default)]
pub struct CovListTraceToVec {
    pub trace: crate::covlist::CovList,

    last_bb_start_pc: Option<u64>,
}

#[cfg(feature = "alloc")]
impl wasm::config::Config for CovListTraceToVec {
    fn instruction_hook(&mut self, bytecode: &[u8], pc: usize) {
        use wasm::opcode::*;
        let last_bb_start_pc = self.last_bb_start_pc.unwrap_or({
            let instr = bytecode[pc];
            trace!("entering basic block with pc = {pc:#x?}, instruction = {instr:#02x?}");
            pc.try_into().unwrap()
        });
        self.last_bb_start_pc = Some(last_bb_start_pc);

        if let instr @ (UNREACHABLE | LOOP | IF | ELSE | END | BR | BR_IF | BR_TABLE | RETURN
        | CALL | CALL_INDIRECT) = bytecode[pc]
        {
            let last_bb_end_pc: u64 = pc.try_into().unwrap();
            self.trace.insert(last_bb_start_pc..(last_bb_end_pc + 1));
            trace!("leaving basic block with pc = {pc:#x?}, instruction = {instr:#02x?}");
        }
    }
}

pub struct CovListTraceToVecIterator<'a>(
    core::iter::Flatten<crate::covlist::CovListRefIterator<'a>>,
);

impl<'a> Iterator for CovListTraceToVecIterator<'a> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'a> IntoIterator for &'a CovListTraceToVec {
    type Item = u64;

    type IntoIter = CovListTraceToVecIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        CovListTraceToVecIterator((&self.trace).into_iter().flatten())
    }
}

impl ExecutionTrace for CovListTraceToVec {
    fn covers(&self, instr_addr: u64) -> bool {
        self.trace.contains(instr_addr)
    }
}
