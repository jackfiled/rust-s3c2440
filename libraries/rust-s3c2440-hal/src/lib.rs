#![cfg_attr(not(test), no_std)]
#![allow(dead_code)]

use core::arch::asm;

pub mod gpio;
pub mod nand;
pub mod uart;
mod utils;

pub use utils::Global;

#[inline(always)]
pub fn nop() {
    unsafe {
        asm!("nop", options(nomem, nostack, preserves_flags));
    }
}
