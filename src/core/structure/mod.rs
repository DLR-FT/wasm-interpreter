//! # Abstract Syntax
//!
//! The Wasm specification defines an abstract syntax that is produced through the processes of
//! decoding and parsing[^structure]. Afterwards, this abstract syntax is validated according to
//! [^validation].
//!
//! However, this interpreter combines the decoding and validation phases into one function
//! [`decode_and_validate`](crate::decode_and_validate). Therefore, we are not bound to produce the
//! same, potentially invalid, intermediate abstract syntax that is defined in [^structure], but
//! rather a more specific version of it that is always valid.
//!
//! This module defines such a valid abstract syntax. Note that in most places our valid abstract
//! syntax uses the same definitions from the official abstract syntax, with validation invariants
//! documented through Rust's safety mechanisms (to be relied on later during execution).
//!
//! [^structure]: [WebAssembly Specification 2.0 - 2. Structure](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#structure%E2%91%A0)
//! [^validation]: [WebAssembly Specification 2.0 - 3. Validation](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/#validation%E2%91%A1)

pub mod instructions;
pub mod modules;
pub mod types;
// TODO this module technically belongs to validation
pub mod import_subtyping;
