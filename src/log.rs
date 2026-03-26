use core::fmt::{self, Write};

use crate::sync::Mutex;

/// Log levels of the kernel logger.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[allow(unused)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

struct Logger {
    level: LogLevel,
}

impl Write for Logger {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // TODO: direct log output to other places
        crate::console::write(s);
        Ok(())
    }
}

static LOGGER: Mutex<Logger> = Mutex::new(Logger {
    level: LogLevel::Debug,
});

/// Sets the log level of the kernel logger. After calling this function, subsequent logged messages
/// will be discarded if their level is below the new setting.
pub fn set_level(level: LogLevel) {
    LOGGER.with_locked(|logger| logger.level = level);
}

#[doc(hidden)]
pub fn _log(level: LogLevel, args: fmt::Arguments) {
    // TODO: Use scheduler time once that's implemented
    let time: u32 = 0;
    let secs = time / 1000;
    let millis = time % 1000;

    LOGGER.with_locked(|logger| {
        if level >= logger.level {
            write!(logger, "[{:5}.{:03}] {}\n", secs, millis, args).unwrap();
        }
    });
}

/// Writes a formatted message to the kernel log at the `Debug` level.
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => (crate::log::_log(crate::log::LogLevel::Debug, format_args!($($arg)*)));
}

/// Writes a formatted message to the kernel log at the `Info` level.
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => (crate::log::_log(crate::log::LogLevel::Info, format_args!($($arg)*)));
}

/// Writes a formatted message to the kernel log at the `Warning` level.
#[macro_export]
macro_rules! log_warning {
    ($($arg:tt)*) => (crate::log::_log(crate::log::LogLevel::Warning, format_args!($($arg)*)));
}

/// Writes a formatted message to the kernel log at the `Error` level.
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => (crate::log::_log(crate::log::LogLevel::Error, format_args!($($arg)*)));
}
