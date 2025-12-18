use crate::s3c2440::CLOCK_CONTROLLER;
use crate::singleton;
use crate::utils::Register;
use bitflags::bitflags;
use core::ops::Deref;

bitflags! {
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub struct ClockStatus : u32 {
        const AC97 = 1 << 20;
        const Camera = 1 << 19;
        const SPI = 1 << 18;
        const IIS = 1 << 17;
        const IIC = 1 << 16;
        const ADC = 1 << 15;
        const RTC = 1 << 14;
        const GPIO = 1 << 13;
        const UART2 = 1 << 12;
        const UART1 = 1 << 11;
        const UART0 = 1 << 10;
        const SDI = 1 << 9;
        const PWMTIMER = 1 << 8;
        const USB_DEVICE = 1 << 7;
        const USB_HOST = 1 << 6;
        const LCDC = 1 << 5;
        const NAND_FLASH_CONTROLLER = 1 << 4;
        const SLEEP = 1 << 3;
        const IDLE_BIT = 1 << 2;
        // 位 [1:0] 保留，不定义为独立标志
    }
}

#[repr(C)]
pub struct ClockControllerInner {
    clock_register: Register,
    slow_clock_register: Register,
}

pub struct ClockToken {
    released: bool,
    clock: ClockStatus,
}

impl ClockToken {
    pub fn release(mut self) {
        self.do_release()
    }

    pub fn status(&self) -> &ClockStatus {
        &self.clock
    }

    fn do_release(&mut self) {
        if !self.released {
            let controller = ClockController::new();
            controller.close_clock(self.clock)
        }
    }
}

impl Drop for ClockToken {
    fn drop(&mut self) {
        self.do_release();
    }
}

impl ClockControllerInner {
    #[inline]
    pub fn open_clock(&self, clock: ClockStatus) -> ClockToken {
        self.clock_register
            .write(self.clock_register.read() | clock.bits());

        ClockToken {
            released: false,
            clock,
        }
    }

    #[inline]
    fn close_clock(&self, clock: ClockStatus) {
        self.clock_register
            .write(self.clock_register.read() & (!clock.bits()));
    }
}

pub struct ClockController {
    inner: *const ClockControllerInner,
}

impl Deref for ClockController {
    type Target = ClockControllerInner;

    fn deref(&self) -> &Self::Target {
        unsafe { &(*self.inner) }
    }
}

impl ClockController {
    fn new() -> Self {
        Self {
            inner: CLOCK_CONTROLLER as *const ClockControllerInner,
        }
    }
}

/// Get the mutable clock controller.
/// Only called once.
pub fn get_clock_controller() -> &'static mut ClockController {
    singleton!(:ClockController = ClockController::new()).unwrap()
}

#[cfg(test)]
mod tests {
    use crate::clock::ClockStatus;

    #[test]
    fn clock_status_tests() {
        assert_eq!(0x20000, ClockStatus::IIS.bits());
    }
}
