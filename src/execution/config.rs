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
}

/// Default implementation of the interpreter configuration, with all hooks empty
impl Config for () {}
