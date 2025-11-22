#![no_std]
#![no_main]

use rust_s3c2440_std::{entry, println};

#[entry]
fn main() -> ! {
    println!("Hello from flash-reader application.");
    loop {}
}
