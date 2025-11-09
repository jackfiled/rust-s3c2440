#![no_std]
#![no_main]

use rust_s3c2440_base::system::PCLK;
use rust_s3c2440_library::hardware::{S3C2440UartController, nop};

#[unsafe(no_mangle)]
fn rust_main() -> ! {
    let uart_controller = S3C2440UartController::uart_controller0();
    uart_controller.initialize(PCLK, 115200);

    for c in b"Hello from Rust!" {
        uart_controller.try_write(core::slice::from_ref(c));
        nop();
    }
    loop {}
}
