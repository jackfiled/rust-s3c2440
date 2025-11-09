mod console;
#[macro_use]
mod logger;

pub use console::S3C2440Console;
pub use logger::{Log, LogLevel, get_logger, set_logger};
