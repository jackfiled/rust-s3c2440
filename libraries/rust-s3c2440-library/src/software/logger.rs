use crate::Global;
use core::fmt::{Arguments, Display, Formatter};

#[derive(Copy, Clone, Debug)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            LogLevel::Debug => f.write_str("DEBUG"),
            LogLevel::Info => f.write_str("INFO"),
            LogLevel::Warn => f.write_str("WARN"),
            LogLevel::Error => f.write_str("ERROR"),
        }
    }
}

pub trait Log {
    fn log(&self, level: LogLevel, agrs: Arguments);
}

struct NopLogger;

impl Log for NopLogger {
    fn log(&self, _: LogLevel, _: Arguments) {}
}

struct LogContext {
    logger: &'static dyn Log,
    log_level: LogLevel,
}

static CONTEXT: Global<LogContext> = Global::new();

pub static mut LOGGER: &dyn Log = &NopLogger;

pub fn set_logger(logger: &'static dyn Log, level: LogLevel) {
    CONTEXT.init(LogContext {
        logger,
        log_level: level,
    });
}

pub fn get_logger() -> &'static dyn Log {
    unsafe { CONTEXT.get_unchecked().logger }
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        use core::fmt::Write;
        unsafe {
            $crate::software::get_logger().log(LogLevel::Debug, core::format_args!($($arg)*));
        }
    }
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        use core::fmt::Write;
        unsafe {
            $crate::software::get_logger().log(LogLevel::Info, core::format_args!($($arg)*));
        }
    }
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        use core::fmt::Write;
        unsafe {
            $crate::software::get_logger().log(LogLevel::Warn, core::format_args!($($arg)*));
        }
    }
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        use core::fmt::Write;
        unsafe {
            $crate::software::get_logger().log(LogLevel::Error, core::format_args!($($arg)*));
        }
    }
}
