use crate::Runner;

/// Trait that allows user specified hooks for various events during interpretation
///
/// The default implementation of all trait methods are empty, i. e. can be optimized out fully.
// It mus always be checked that there is no performance penalty for an empty hook!
pub trait HookSet: Default {
    /// A hook which is called before every wasm instruction
    ///
    /// This allows the most intricate insight into the interpreters behavior, at the cost of a
    /// hefty performance penalty
    #[allow(unused_variables)]
    fn instruction_hook(interpreter_state: &mut Runner<Self>) {}
}

/// Default implementation of a hookset, with all hooks empty
#[derive(Default)]
pub struct EmptyHookSet;

impl HookSet for EmptyHookSet {}
