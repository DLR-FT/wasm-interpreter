/// Trait that allows user specified configuration for various items during interpretation. Additionally, the types
/// implementing this trait can act as custom user data within an interpreter instance, passed along to each method of
/// this trait and host functions whenever they are invoked.
///
/// The default implementation of all trait methods have the least overhead, i. e. most can be optimized out fully.
// It must always be checked that there is no additional performance penalty for the default config!
pub trait Config {
    /// Maximum number of values in the value stack
    const MAX_VALUE_STACK_SIZE: usize = 0xf0000; // 64 Kibi-Values

    /// Maximum number of cascading function invocations
    const MAX_CALL_STACK_SIZE: usize = 0x1000; // 4 Kibi-Functions

    /// A hook which is called before every wasm instruction
    ///
    /// This allows the most intricate insight into the interpreters behavior, at the cost of a
    /// hefty performance penalty
    #[allow(unused_variables)]
    #[inline(always)]
    fn instruction_hook(&mut self, bytecode: &[u8], pc: usize) {}

    /// Amount of fuel to be deducted when a single byte `instr` is hit. The cost corresponding to `UNREACHABLE` and
    /// `END` instructions and other bytes that do not correspond to any Wasm instruction are ignored.
    // It must always be checked that the calls to this method fold into a constant if it is just a match statement that
    // yields constants.
    #[inline(always)]
    fn get_flat_cost(_instr: u8) -> u32 {
        1
    }

    /// Amount of fuel to be deducted when a multi-byte instruction that starts with the byte 0xFC is hit. This method
    /// should return the cost of an instruction obtained by prepending 0xFC to of an unsigned 32-bit LEB
    /// representation of `instr`. Multi-byte sequences obtained this way that do not correspond to any Wasm instruction
    /// are ignored.
    // It must always be checked that the calls to this method fold into a constant if it is just a match statement that
    // yields constants.
    #[inline(always)]
    fn get_fc_extension_flat_cost(_instr: u32) -> u32 {
        1
    }

    /// Amount of fuel to be deducted when a multi-byte instruction that starts with the byte 0xFD is hit. This method
    /// should return the cost of an instruction obtained by prepending 0xFD to of an unsigned 32-bit LEB
    /// representation of `instr`. Multi-byte sequences obtained this way that do not correspond to any Wasm instruction
    /// are ignored.
    // It must always be checked that the calls to this method fold into a constant if it is just a match statement that
    // yields constants.
    #[inline(always)]
    fn get_fd_extension_flat_cost(_instr: u32) -> u32 {
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
    fn get_cost_per_element(_instr: u8) -> u32 {
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
    fn get_fc_extension_cost_per_element(_instr: u32) -> u32 {
        0
    }
}

/// Default implementation of the interpreter configuration, with all hooks empty
impl Config for () {}
