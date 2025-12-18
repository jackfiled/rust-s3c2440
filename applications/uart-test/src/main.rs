#![no_std]
#![no_main]

use rust_s3c2440_hal::gpio::{PortHPin2, PortHPin3};
use rust_s3c2440_hal::nop;
use rust_s3c2440_hal::s3c2440::PCLK;
use rust_s3c2440_hal::uart::S3C2440UartControllerBuilder;
use rust_s3c2440_std::entry;
use rust_s3c2440_std::system::clock::delay_ms;

/// This is a very special binary application.
/// This application uses the minimum abstractions provided by the library to test weather we
/// can run an application written in Rust on the S3C2440.
#[entry(call_init = false)]
fn main() -> ! {
    let builder = S3C2440UartControllerBuilder::uart_controller0(
        PortHPin2::init().into_uart_transmit(),
        PortHPin3::init().into_uart_receive(),
    );

    let controller = builder.build_fifo::<PCLK, 115200>();

    for _ in 0..100 {
        controller.try_write(b"Hello, TQ2440!\r\n");
        nop();
    }

    loop {
        delay_ms(1000);
    }
}
