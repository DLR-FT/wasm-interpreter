//! A function to validate every section type. These functions may also extract and return
//! relevant information.
//!
//! Note that some implementations are exactly equal to those in [crate::execution::sections].
//! They were not generalized on purpose for a simpler project structure.
pub use code::*;
pub use export::*;
pub use function::*;
pub use import::*;
pub use memory::*;
pub use r#type::*;
pub use table::*;
pub use global::*;
pub use start::*;

mod code;
mod export;
mod function;
mod import;
mod memory;
mod table;
mod r#type;
mod global;
mod start;
