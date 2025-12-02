#![no_std]
#![no_main]

use rust_s3c2440_hal::gpio::{PortHPin2, PortHPin3};
use rust_s3c2440_hal::nop;
use rust_s3c2440_hal::uart::S3C2440UartControllerBuilder;
use rust_s3c2440_std::entry;
use rust_s3c2440_std::system::PCLK;

/// This is a very special binary application.
/// This application uses the minimum abstractions provided by the library to test weather we
/// can run an application written in Rust on the S3C2440.
#[entry(call_init = false)]
fn main() -> ! {
    let controller = S3C2440UartControllerBuilder::uart_controller0(
        PortHPin2::new().into_uart_transmit(),
        PortHPin3::new().into_uart_receive(),
    )
    .initialize(PCLK, 115200);

    for c in b"Hello from Rust!" {
        controller.try_write(core::slice::from_ref(c));
        nop();
    }

    loop {}
}
