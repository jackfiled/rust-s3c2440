#![no_std]
#![no_main]
#![allow(dead_code)]

use crate::manager::{InitializeConfiguration, Manager};
use core::panic::PanicInfo;

mod manager;
pub mod support;
pub mod system;
mod utils;

pub use manager::MANAGER;

// Make the linker happy, as the rust_main will be defined in application crate.
unsafe extern "C" {
    fn rust_main() -> !;
}

use crate::support::LogLevel;
/// Use this macro to decorate the user main function.
/// Reimport from rust-s3c2440-macros.
pub use rust_s3c2440_macros::entry;

const CONFIGURATION: InitializeConfiguration = InitializeConfiguration {
    uart_port: 0,
    uart_buad_rate: 115200,
    log_level: LogLevel::Debug,
};

/// Hook function will be called before entry function running.
pub fn init_board() {
    Manager::initialize(CONFIGURATION);
    info!("Board manager is initialized.");
}

#[panic_handler]
pub fn panic_handler(info: &PanicInfo) -> ! {
    // The default formatter may print too long line, which breaks the UART?
    error!("System panicked: {}", info.message());
    if let Some(l) = info.location() {
        error!("Location: {}", l);
    }

    loop {}
}
