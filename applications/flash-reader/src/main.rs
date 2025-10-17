#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[unsafe(export_name = "useAppInit")]
fn main() -> i32 {
    0
}

#[panic_handler]
fn handle_panic(_: &PanicInfo) -> ! {
    loop {}
}
