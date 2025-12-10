#![cfg_attr(not(test), no_std)]
#![allow(dead_code)]

use core::arch::asm;

pub mod clock;
pub mod dma;
pub mod gpio;
pub mod iis;
pub mod interrupt;
pub mod l3bus;
pub mod nand;
pub mod s3c2440;
pub mod uart;
pub mod utils;

pub use utils::Global;

#[inline(always)]
pub fn nop() {
    unsafe { asm!("nop", options(nomem, nostack, preserves_flags)) }
}

pub fn delay_cycles(mut cycles: u32) {
    // Group 8 nops once to decrease the cost of loop.
    while cycles > 8 {
        unsafe {
            asm!(
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                options(nomem, nostack, preserves_flags)
            );
        }

        cycles -= 8;
    }

    while cycles > 8 {
        nop();
        cycles -= 1;
    }
}

fn calculate_main_clock(m: u32, p: u32, s: u32, input_clock: u32) -> u32 {
    2 * (m + 8) * input_clock / ((p + 2) * 2u32.pow(s))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main_clock_tests() {
        assert_eq!(210, calculate_main_clock(97, 1, 2, 12));
        // Wired `MPLL` configuration comes from IIS device code.
        assert_eq!(393, calculate_main_clock(123, 6, 0, 12));
        assert_eq!(406, calculate_main_clock(229, 5, 1, 12));
    }
}
