#![no_std]
#![no_main]

use rust_s3c2440_hal::nand::{NandFlashController, NandFlashControllerBuilder};
use rust_s3c2440_std::system::clock::delay_ms;
use rust_s3c2440_std::{entry, println};

#[entry]
fn main() -> ! {
    let controller = NandFlashControllerBuilder::build();

    println!(
        "Nand flash initialized with device ID {:#x}",
        controller.device_id()
    );

    // Try to write and read one block.
    const TARGET_BLOCK: usize = 2025;
    const DATA: &str = "Cross the great wall, come to the world.";
    let address = ((TARGET_BLOCK - 1) * NandFlashController::BLOCK_SIZE).into();

    // println!("Writing block...");
    // controller.write(address, DATA.as_bytes()).unwrap();
    // if controller.write(address, DATA.as_bytes()).is_err() {
    //     println!("Failed to write!");
    //     loop {}
    // }

    println!("Reading block...");
    let mut buffer = [0u8; DATA.len()];

    controller.read(address, &mut buffer).unwrap();

    match core::str::from_utf8(&buffer) {
        Ok(s) => {
            println!("The reading result is '{s}'.");
        }
        Err(e) => {
            println!("Failed to parse buffer as string: {e}.");
        }
    }

    loop {
        delay_ms(1000);
    }
}
