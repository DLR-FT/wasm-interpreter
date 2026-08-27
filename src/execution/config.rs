use crate::DispatchMechanism;

/// Trait that allows user specified configuration for various items during interpretation. Additionally, the types
/// implementing this trait can act as custom user data within an interpreter instance, passed along to each method of
/// this trait and host functions whenever they are invoked.
///
/// The default implementation of all trait methods have the least overhead, i. e. most can be optimized out fully.
// It must always be checked that there is no additional performance penalty for the default config!
pub trait Config {
    /// Maximum number of values in the value stack
    const MAX_VALUE_STACK_SIZE: usize = 0x10000; // 64 Kibi-Values

    /// Maximum number of cascading function invocations
    const MAX_CALL_STACK_SIZE: usize = 0x1000; // 4 Kibi-Functions

    /// An optional size limit (i.e. capacity) for table sizes. If set, the table is only allocated
    /// once with that capacity, but all table grows exceeding that capacity will fail.
    ///
    /// This is essentially a weaker version of [`Config::memory_requested_allocation`], as tables
    /// allocation management likely does not need as much flexibility relative to memories.
    /// However, a `Config::table_requested_allocation` may be added in the future.
    const TABLE_CAPACITY: Option<usize> = None;

    /// The mechanism to use for dispatching during interpretation. Refer to [`DispatchMechanism`]
    /// for a list of all mechanisms, including their up- and downsides.
    const DISPATCH_MECHANISM: DispatchMechanism = DispatchMechanism::LoopCall;

    /// A function to manage memory allocation requests by linear memories.
    ///
    /// It is called whenever it is determined that a new memory allocation or a reallocation must
    /// take place, as part of either instantiation or growing memory. Note that calls to this
    /// function do not necessarily map one-to-one to memory grows, as an existing allocation may
    /// suffice for growing a memory to succeed.
    ///
    /// When given information about the memory allocation, this function must return the number of
    /// Wasm pages that will be allocated in the end. If this is `None` or the allocation size in
    /// bytes overflows, the original memory operation may fail with
    /// [`RuntimeError::HostRefusedAllocation`](crate::RuntimeError::HostRefusedAllocation) or
    /// [`RuntimeError::MemoryOverflowed`](crate::RuntimeError::MemoryOverflowed), respectively.
    ///
    /// # Arguments
    ///
    /// - `current_page_capacity` - The number of Wasm pages currently allocated to this memory.
    ///   This can be `None`, if the memory does not have a backing allocation yet.
    /// - `additional_page_capacity` - The minimum number of Wasm pages by which the current
    ///   allocation should be grown (or a new one allocated).
    /// - `memory_upper_limit` - The upper page count limit of the memory type. This can be `None`,
    ///   if no upper limit is specified.
    ///
    /// It is guaranteed that `current_page_capacity.unwrap_or(0) + additional_page_capacity` is
    /// less or equal to the upper limit, if one is present. This is because allocating or growing a
    /// memory would have failed early, if its upper limit is exceeded.
    #[inline(always)]
    fn memory_requested_allocation(
        current_page_capacity: Option<usize>,
        additional_page_capacity: usize,
        memory_upper_limit: Option<usize>,
    ) -> Option<usize> {
        // Default behavior is independent of these
        let _ = current_page_capacity;
        let _ = memory_upper_limit;

        // Allocates, then possibly reallocates later
        Some(additional_page_capacity)
    }

    /// A function to manage memory allocation requests by tables.
    ///
    /// This works exactly the same as [`Config::memory_requested_allocation`] which is used for
    /// memories instead of tables.
    #[inline(always)]
    fn table_requested_allocation(
        current_capacity: Option<usize>,
        additional_capacity: usize,
        table_upper_limit: Option<usize>,
    ) -> Option<usize> {
        // Default behavior is independent of these
        let _ = current_capacity;
        let _ = table_upper_limit;

        // Allocates, then possibly reallocates later
        Some(additional_capacity)
    }

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

// TODO remove. This is an initial PoC for memory resource limiter usage
pub struct MinimalAllocationsConfig;
impl Config for MinimalAllocationsConfig {
    #[inline(always)]
    fn memory_requested_allocation(
        current_page_capacity: Option<usize>,
        additional_page_capacity: usize,
        memory_upper_limit: Option<usize>,
    ) -> Option<usize> {
        // Never realloc
        if current_page_capacity.is_some() {
            return None;
        }

        // Always allocate with maximum size, or if not present the requested size.
        Some(memory_upper_limit.unwrap_or(additional_page_capacity))
    }
}
