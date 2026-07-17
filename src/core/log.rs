#[macro_export]
macro_rules! error {
    ($($arg:tt)+) => {
        #[cfg(feature = "log")]
        ::log::error!($($arg)+);
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)+) => {
        #[cfg(feature = "log")]
        ::log::warn!($($arg)+);
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)+) => {
        #[cfg(feature = "log")]
        ::log::info!($($arg)+);
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)+) => {
        #[cfg(feature = "log")]
        ::log::debug!($($arg)+);
    };
}

#[macro_export]
macro_rules! trace {
    ($($arg:tt)+) => {
        #[cfg(feature = "log")]
        ::log::trace!($($arg)+);
    };
}
