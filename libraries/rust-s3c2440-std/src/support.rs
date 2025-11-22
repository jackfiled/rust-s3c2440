pub mod console;
mod logger;

use core::error::Error;
use core::fmt::{Display, Formatter};
pub use logger::{Log, LogLevel, get_logger, set_logger};

#[derive(Debug)]
pub struct S3C2440Error {}

impl Display for S3C2440Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "A internal error is occurred.")
    }
}

impl Error for S3C2440Error {}

pub type Result<T> = core::result::Result<T, S3C2440Error>;
