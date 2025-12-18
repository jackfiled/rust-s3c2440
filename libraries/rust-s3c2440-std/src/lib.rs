#![feature(alloc_error_handler)]
#![no_std]
#![no_main]
// Lint configurations
#![allow(dead_code)]
#![allow(static_mut_refs)]
#![allow(asm_sub_register)]

use crate::system::Manager;
use core::panic::PanicInfo;
use log::{error, info};

extern crate alloc;

pub mod audio;
#[macro_use]
pub mod io;
pub mod system;
pub mod utils;

// Make the linker happy, as the rust_main will be defined in application crate.
unsafe extern "C" {
    fn rust_main() -> !;
}

/// Use this macro to decorate the user main function.
/// Reimport from rust-s3c2440-macros.
pub use rust_s3c2440_macros::entry;

/// Hook function will be called before entry function running.
pub fn init_board() {
    Manager::initialize();
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
