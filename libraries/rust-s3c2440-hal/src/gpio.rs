mod port;
mod register;
mod state;

use core::error::Error;
use core::fmt::{Display, Formatter};
use embedded_hal::digital::ErrorKind;
pub use port::*;
pub use state::*;

#[derive(Debug)]
pub struct PinError {
    msg: &'static str,
}

impl Display for PinError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "GPIO Error: {}", self.msg)
    }
}

impl Error for PinError {}

impl embedded_hal::digital::Error for PinError {
    fn kind(&self) -> ErrorKind {
        ErrorKind::Other
    }
}
