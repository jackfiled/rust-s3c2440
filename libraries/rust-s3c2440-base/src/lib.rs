#![no_std]
#![no_main]

use crate::manager::{InitializeConfiguration, Manager};
use core::panic::PanicInfo;
use rust_s3c2440_library::software::LogLevel;

#[macro_use]
extern crate rust_s3c2440_library;

mod manager;
pub mod system;
mod utils;

pub use manager::MANAGER;

// Make the linker happy, as the rust_main will be defined in application crate.
unsafe extern "C" {
    fn rust_main() -> !;
}

/// Use this macro to decorate the user main function.
#[macro_export]
macro_rules! entry {
    ($path:path) => {
        #[unsafe(no_mangle)]
        pub fn rust_main() -> ! {
            $crate::init_board();
            $path()
        }
    };
}

/// Hook function will be called before entry function running.
pub fn init_board() {
    let mut manager = Manager::new();
    manager.initialize(&InitializeConfiguration {
        uart_port: 0,
        uart_buad_rate: 115200,
        log_level: LogLevel::Info,
    });
    MANAGER.init(manager);
    info!("Board manager is initialized.");

    println!("Hello from Rust!");
}

#[panic_handler]
pub fn panic_hanlder(_: &PanicInfo) -> ! {
    loop {}
}
