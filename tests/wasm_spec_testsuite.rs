// The reason this file exists is only to expose the `specification` module to the outside world.
// More so, the reason it wasn't added to the `lib.rs` file is because we wanted to separate the
// regular tests from the spec tests.
#[cfg(feature = "spec-test")]
mod specification;
