//! Helpers for assertions due to prior validation of a WASM program.

use core::fmt::Debug;

pub(crate) trait UnwrapValidatedExt<T> {
    fn unwrap_validated(self) -> T;
}

impl<T> UnwrapValidatedExt<T> for Option<T> {
    /// Indicate that we can assume this Option to be Some(_) due to prior validation
    fn unwrap_validated(self) -> T {
        self.expect("Validation guarantees this to be `Some(_)`, but it is `None`")
    }
}

impl<T, E: Debug> UnwrapValidatedExt<T> for Result<T, E> {
    /// Indicate that we can assume this Result to be Ok(_) due to prior validation
    fn unwrap_validated(self) -> T {
        self.unwrap_or_else(|e| {
            panic!("Validation guarantees this to be `Ok(_)`, but it is `Err({e:?})`");
        })
    }
}

#[macro_export]
macro_rules! unreachable_validated {
    () => {
        unreachable!("because of prior validation")
    };
}
