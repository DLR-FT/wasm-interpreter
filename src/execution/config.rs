/// Trait that allows user specified configuration for various items during interpretation
///
/// The default implementation of all trait methods have the least overhead, i. e. most can be optimized out fully.
// It must always be checked that there is no additional performance penalty for the default config!
pub trait Config: Default {
    /// A hook which is called before every wasm instruction
    ///
    /// This allows the most intricate insight into the interpreters behavior, at the cost of a
    /// hefty performance penalty
    #[allow(unused_variables)]
    fn instruction_hook(&mut self, bytecode: &[u8], pc: usize) {}
}

/// Default implementation of the interpreter configuration, with all hooks empty
#[derive(Default, Debug)]
pub struct DefaultConfig;

impl Config for DefaultConfig {}
