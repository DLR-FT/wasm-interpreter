#![no_std]

#[cfg(feature = "log")]
pub use ::log::{debug, error, info, trace, warn};

#[cfg(not(feature = "log"))]
/// Noop, expands to nothing
#[macro_export]
macro_rules! error {
    ($($arg:tt)+) => {{}};
}

#[cfg(not(feature = "log"))]
/// Noop, expands to nothing
#[macro_export]
macro_rules! warn {
    ($($arg:tt)+) => {{}};
}

#[cfg(not(feature = "log"))]
/// Noop, expands to nothing
#[macro_export]
macro_rules! info {
    ($($arg:tt)+) => {{}};
}

#[cfg(not(feature = "log"))]
/// Noop, expands to nothing
#[macro_export]
macro_rules! debug {
    ($($arg:tt)+) => {{}};
}

#[cfg(not(feature = "log"))]
/// Noop, expands to nothing
#[macro_export]
macro_rules! trace {
    ($($arg:tt)+) => {{}};
}
