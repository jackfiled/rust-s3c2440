mod gpio;
mod uart;

use core::arch::asm;
pub use gpio::GPIOController;
pub use uart::S3C2440UartController;

/// A helper function to generate one no-operation instruction.
/// The generated instruction is `mov r0, r0`.
#[inline(always)]
pub fn nop() {
    unsafe {
        asm!("nop", options(nomem, nostack, preserves_flags));
    }
}