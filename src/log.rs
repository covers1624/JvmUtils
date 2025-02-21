#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        if cfg!(feature = "logging") {
            log::error!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        if cfg!(feature = "logging") {
            log::warn!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        if cfg!(feature = "logging") {
            log::info!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        if cfg!(feature = "logging") {
            log::debug!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! log_trace {
    ($($arg:tt)*) => {
        if cfg!(feature = "logging") {
            log::debug!($($arg)*);
        }
    };
}
