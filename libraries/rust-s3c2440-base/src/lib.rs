#![no_std]
#![no_main]

use crate::system::PCLK;
use core::panic::PanicInfo;
use rust_s3c2440_library::gpio::GPIOController;
use rust_s3c2440_library::uart::S3C2440UartController;

mod system;

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
    // Initialize GPIO controller.
    let gpio_controller = GPIOController::new();
    gpio_controller.initialize();

    // Initialize UART port.
    let uart_controller = S3C2440UartController::uart_controller0();
    uart_controller.initialize(PCLK, 115200);

    uart_controller.write(b"Hello from Rust!");
}

#[panic_handler]
pub fn panic_hanlder(_: &PanicInfo) -> ! {
    loop {}
}
