//! A function to extract information from most section type. These functions may also extract and return
//! relevant information.
//!
//! Note that some implementations are exactly equal to those in [crate::validation::sections].
//! They were not generalized on purpose for a simpler project structure.
pub mod export;
pub mod function;
pub mod import;
pub mod memory;
pub mod table;
pub mod r#type;
pub mod global;
