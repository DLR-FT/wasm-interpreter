use core::fmt::{Debug, Display};
use core::hint::unreachable_unchecked;

/// A helper trait to unwrap Options/Results that are expected to be Some/Ok due to prior validation.
pub(crate) trait UnwrapValidatedExt<T> {
    fn unwrap_validated(self) -> T;
}

impl<T> UnwrapValidatedExt<T> for Option<T> {
    fn unwrap_validated(self) -> T {
        self.expect("this to be `Some` because of prior validation")
    }
}

impl<T, E: Debug> UnwrapValidatedExt<T> for Result<T, E> {
    fn unwrap_validated(self) -> T {
        self.expect("this to be `Ok` because of prior validation")
    }
}

#[macro_export]
macro_rules! unreachable_validated {
    () => {
        unreachable!("because of prior validation")
    };
}
