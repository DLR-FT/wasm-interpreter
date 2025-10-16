#![no_std]

#[cfg(feature = "log")]
pub use ::log::{debug, error, info, trace, warn};

#[cfg(not(feature = "log"))]
mod log_noop {
    /// Noop, expands to nothing
    #[macro_export]
    macro_rules! error {
        ($($arg:tt)+) => {{}};
    }

    /// Noop, expands to nothing
    #[macro_export]
    macro_rules! warn {
        ($($arg:tt)+) => {{}};
    }

    /// Noop, expands to nothing
    #[macro_export]
    macro_rules! info {
        ($($arg:tt)+) => {{}};
    }

    /// Noop, expands to nothing
    #[macro_export]
    macro_rules! debug {
        ($($arg:tt)+) => {{}};
    }

    /// Noop, expands to nothing
    #[macro_export]
    macro_rules! trace {
        ($($arg:tt)+) => {{}};
    }
}
