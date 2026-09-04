use core::num::{NonZeroU16, NonZeroUsize};

use crate::DispatchMechanism;

/// Trait that allows user specified configuration for various items during interpretation. Additionally, the types
/// implementing this trait can act as custom user data within an interpreter instance, passed along to each method of
/// this trait and host functions whenever they are invoked.
///
/// The default implementation of all trait methods have the least overhead, i. e. most can be optimized out fully.
// It must always be checked that there is no additional performance penalty for the default config!
pub trait Config {
    /// Maximum number of values in the value stack
    ///
    /// The default of `0xd0`/`208` is just the bare minimum multiple of 16 to pass the Specification Test Suite
    const MAX_VALUE_STACK_SIZE: usize = 0xd0;

    /// Maximum number of cascading function invocations
    ///
    /// The default of `0xd0`/`208` is just the bare minimum multiple of 16 to pass the Specification Test Suite
    const MAX_CALL_STACK_SIZE: usize = 0xd0;

    /// An optional limit for the number of pages a memory's size can grow to.
    // TODO(memory64): Use Option<NonZeroUsize> with limit of 2^48 pages
    // TODO(custom-page-sizes): Use Option<NonZeroU32>
    const MAX_NUMBER_OF_MEMORY_PAGES: Option<NonZeroU16> = None;

    /// An optional limit for the number of elements a table's size can grow to.
    const MAX_NUMBER_OF_TABLE_ELEMENTS: Option<NonZeroUsize> = None;

    /// The mechanism to use for dispatching during interpretation. Refer to [`DispatchMechanism`]
    /// for a list of all mechanisms, including their up- and downsides.
    const DISPATCH_MECHANISM: DispatchMechanism = DispatchMechanism::LoopCall;

    /// A hook which is called before every wasm instruction
    ///
    /// This allows the most intricate insight into the interpreters behavior, at the cost of a
    /// hefty performance penalty
    #[inline(always)]
    fn instruction_hook(&mut self, _bytecode: &[u8], _pc: usize) {}

    /// Amount of fuel to be deducted when a single byte `instr` is hit. The cost corresponding to `UNREACHABLE` and
    /// `END` instructions and other bytes that do not correspond to any Wasm instruction are ignored.
    // It must always be checked that the calls to this method fold into a constant if it is just a match statement that
    // yields constants.
    #[inline(always)]
    fn get_flat_cost(_instr: u8) -> u64 {
        1
    }

    /// Amount of fuel to be deducted when a multi-byte instruction that starts with the byte 0xFC is hit. This method
    /// should return the cost of an instruction obtained by prepending 0xFC to of an unsigned 32-bit LEB
    /// representation of `instr`. Multi-byte sequences obtained this way that do not correspond to any Wasm instruction
    /// are ignored.
    // It must always be checked that the calls to this method fold into a constant if it is just a match statement that
    // yields constants.
    #[inline(always)]
    fn get_fc_extension_flat_cost(_instr: u32) -> u64 {
        1
    }

    /// Amount of fuel to be deducted when a multi-byte instruction that starts with the byte 0xFD is hit. This method
    /// should return the cost of an instruction obtained by prepending 0xFD to of an unsigned 32-bit LEB
    /// representation of `instr`. Multi-byte sequences obtained this way that do not correspond to any Wasm instruction
    /// are ignored.
    // It must always be checked that the calls to this method fold into a constant if it is just a match statement that
    // yields constants.
    #[inline(always)]
    fn get_fd_extension_flat_cost(_instr: u32) -> u64 {
        1
    }

    /// Amount of fuel to be deducted per element of a single byte instruction `instr` that executes in asymptotically
    /// linear time with respect to one of the values it pops from the stack.
    ///
    /// In Wasm 2.0 specification, this applies to the following instructions:
    /// - `MEMORY.GROW` of type `[n: i32] -> [i32]`
    ///
    /// The cost of the instruction is calculated as `cost := get_flat_cost(instr) + n*get_cost_per_element(instr)`.
    /// where `n` is the stack value marked in the instruction type signature above. Other instructions and bytes that
    /// do not correspond to any instruction are ignored.
    // It must always be checked that the calls to this method fold into a constant if it is just a match statement that
    // yields constants.
    #[inline(always)]
    fn get_cost_per_element(_instr: u8) -> u64 {
        0
    }

    /// Amount of fuel to be deducted per element of a  multi-byte instruction that starts with the byte 0xFC,
    /// which executes in asymptotically linear time with respect to one of the values it pops from the stack. This
    /// method should return the cost of an instruction obtained by prepending 0xFD to of an unsigned 32-bit LEB
    /// representation of `instr`. Multi-byte sequences obtained this way that do not correspond to any Wasm instruction
    /// are ignored.
    ///
    /// In Wasm 2.0 specification, this applies to the following instructions:
    /// - `MEMORY.INIT x`  of type `[d:i32 s: i32 n: i32] -> []`
    /// - `MEMORY.FILL`    of type `[d: i32 val: i32 n: i32] -> []`
    /// - `MEMORY.COPY`    of type `[d: i32 s: i32 n: i32] -> []`
    /// - `TABLE.GROW x`   of type `[val: ref n: i32] -> [i32]`
    /// - `TABLE.INIT x y` of type `[d: i32 s: i32 n: i32] -> []`
    /// - `TABLE.FILL x`   of type `[i: i32 val: ref n: i32] -> []`
    /// - `TABLE.COPY x y` of type `[d: i32 s: i32 n: i32] -> []`
    ///
    /// The cost of the instruction is calculated as `cost := get_flat_cost(instr) + n*get_cost_per_element(instr)`.
    /// where `n` is the stack value marked in the instruction type signature above. Other instructions and multi-byte
    /// sequences that do not correspond to any instruction are ignored.
    // It must always be checked that the calls to this method fold into a constant if it is just a match statement that
    // yields constants.
    #[inline(always)]
    fn get_fc_extension_cost_per_element(_instr: u32) -> u64 {
        0
    }
}

/// Default implementation of the interpreter configuration, with all hooks empty
impl Config for () {}

/// Return the number of bytes consumed for stack memory.
///
/// This includes all bytes allocated for Call-Stack and Value-Stack, including management
/// structures (i.e. the fat pointers holding the allocations of the aforementioned Call- and
/// Value-Stack).
pub const fn stack_memory_bytes_total<T: Config>() -> usize {
    use crate::{
        execution::runtime_structure::value_stack::{CallFrame, Stack},
        Value,
    };

    let value_stack_size = <T as Config>::MAX_VALUE_STACK_SIZE * core::mem::size_of::<Value>();
    let call_stack_size = <T as Config>::MAX_CALL_STACK_SIZE * core::mem::size_of::<CallFrame>();
    let stack_struct_size = core::mem::size_of::<Stack>();

    value_stack_size + call_stack_size + stack_struct_size
}

#[cfg(test)]
mod test {
    use crate::{
        execution::runtime_structure::value_stack::{CallFrame, Stack},
        Value,
    };

    use super::*;

    #[test]
    fn size_of_stack_simple() {
        let management_only_stack_size = core::mem::size_of::<Stack>();
        let default_total_stack_size = stack_memory_bytes_total::<()>();
        let default_data_only_value_stack_size =
            <() as Config>::MAX_VALUE_STACK_SIZE * core::mem::size_of::<Value>();
        let default_data_only_call_stack_size =
            <() as Config>::MAX_CALL_STACK_SIZE * core::mem::size_of::<CallFrame>();

        let default_data_only_stack_size =
            default_data_only_call_stack_size + default_data_only_value_stack_size;
        assert_eq!(
            default_data_only_stack_size as usize,
            default_total_stack_size - management_only_stack_size
        );
    }
}
