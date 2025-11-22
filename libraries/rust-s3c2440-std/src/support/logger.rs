use core::fmt::{Arguments, Display, Formatter};
use rust_s3c2440_hal::Global;

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
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

struct LogContext {
    logger: &'static dyn Log,
    log_level: LogLevel,
}

static CONTEXT: Global<LogContext> = Global::new();

pub fn set_logger(logger: &'static dyn Log, level: LogLevel) {
    CONTEXT.init(LogContext {
        logger,
        log_level: level,
    });
}

pub fn get_logger() -> (&'static dyn Log, LogLevel) {
    let context = unsafe { CONTEXT.get_unchecked() };

    (context.logger, context.log_level)
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        let (logger, log_level) = $crate::support::get_logger();
        if $crate::support::LogLevel::Debug >= log_level {
            logger.log($crate::support::LogLevel::Debug, core::format_args!($($arg)*))
        }
    }
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        let (logger, log_level) = $crate::support::get_logger();
        if $crate::support::LogLevel::Info >= log_level {
            logger.log($crate::support::LogLevel::Info, core::format_args!($($arg)*))
        }
    }
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        let (logger, log_level) = $crate::support::get_logger();
        if $crate::support::LogLevel::Warn >= log_level {
            logger.log($crate::support::LogLevel::Warn, core::format_args!($($arg)*))
        }
    }
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        let (logger, log_level) = $crate::support::get_logger();
        if $crate::support::LogLevel::Error >= log_level {
            logger.log($crate::support::LogLevel::Error, core::format_args!($($arg)*))
        }
    }
}
