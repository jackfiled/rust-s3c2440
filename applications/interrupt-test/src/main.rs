#![no_std]
#![no_main]

use rust_s3c2440_std::system::interrupt::InterruptManager;
use rust_s3c2440_std::system::software_interrupt;
use rust_s3c2440_std::{entry, println};

#[entry]
fn main() -> ! {
    println!("Enable interrupt...");

    let mut interrupt_manager = InterruptManager::new();
    interrupt_manager.enable_interrupt();
    println!("Enabled interrupt.");

    println!("Try to trigger software interrupt...");

    software_interrupt();

    println!("Software interrupt is handled correctly!");

    loop {}
}
