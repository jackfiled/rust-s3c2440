#![no_std]
use core::cell::UnsafeCell;

pub mod gpio;
pub mod uart;

#[repr(transparent)]
pub struct Register(UnsafeCell<u32>);

impl Register {
    /// Read the register.
    pub fn read(&self) -> u32 {
        unsafe { self.0.get().read_volatile() }
    }

    /// Write the register.
    pub fn write(&self, value: u32) {
        unsafe {
            self.0.get().write_volatile(value);
        }
    }
}
