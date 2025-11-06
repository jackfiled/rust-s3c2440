//! This file implements support for the UART controller in S3C2440 SoC which is designed by Samsang.
use crate::Register;
use core::cell::UnsafeCell;

const UART_CONTROLLER0: usize = 0x5000_0000;
const UART_CONTROLLER1: usize = 0x5000_4000;
const UART_CONTROLLER2: usize = 0x5000_8000;

/// The UART buffer register is almost the same as normal register.
/// But it provides 8 bits read/write methods.
#[repr(transparent)]
struct UartBufferRegister(UnsafeCell<u32>);

impl UartBufferRegister {
    fn read(&self) -> u8 {
        unsafe { self.0.get().read_volatile() as _ }
    }

    fn write(&self, value: u8) {
        unsafe {
            self.0.get().write(value as _);
        }
    }
}

#[repr(C)]
struct S3C2440UartControllerInner {
    /// ULCON register.
    line_control: Register,
    /// UCON register.
    control: Register,
    /// UFCON register.
    fifo_control: Register,
    /// UMCON register.
    modem_control: Register,
    /// UTRSTAT register.
    send_receive_status: Register,
    /// UERSTAT register.
    receive_error_status: Register,
    /// UFSTAT regsiter.
    fifo_status: Register,
    /// UMSTAT register.
    modem_status: Register,
    /// UTXH register.
    send_buffer: UartBufferRegister,
    /// URXH register.
    receive_buffer: UartBufferRegister,
    /// UBRDIV register.
    baud_rate_divisor: Register,
}

impl S3C2440UartControllerInner {
    fn init(&self, clock: u32, baud_rate: u32) {
        // Disable fifo.
        self.fifo_control.write(0);

        // Set the line with normal mode, no validation, 1 stop bit and 8 bits length.
        self.line_control.write(0x3);

        // Set the baud rate dividor.
        self.baud_rate_divisor
            .write(Self::calculate_baud_rate_dividor(clock, baud_rate));

        // After setting values, wait for some time.
        for _ in 0..100 {
            core::hint::spin_loop();
        }
    }

    fn read(&self) -> u8 {
        // Wait for receiver buffer ready.
        while !self.is_receive_buffer_empty() {
            core::hint::spin_loop();
        }

        self.receive_buffer.read()
    }

    fn write(&self, buffer: &[u8]) {
        for &c in buffer {
            while !self.is_sender_buffer_empty() {
                core::hint::spin_loop();
            }

            self.send_buffer.write(c);
        }
    }

    fn calculate_baud_rate_dividor(clock: u32, baud_rate: u32) -> u32 {
        // The baud rate dividor should be calculated as follows:
        (clock as f32 / 16.0 / baud_rate as f32 + 0.5) as u32 - 1
    }

    fn is_receive_buffer_empty(&self) -> bool {
        (self.send_receive_status.read() & 0x1) > 0
    }

    fn is_sender_buffer_empty(&self) -> bool {
        (self.send_receive_status.read() & 0x2) > 0
    }

    fn is_sender_ready(&self) -> bool {
        (self.send_receive_status.read() & 0x4) > 0
    }
}

pub struct S3C2440UartController {
    inner: *const S3C2440UartControllerInner,
}

impl S3C2440UartController {
    pub fn uart_controller0() -> Self {
        Self::new(UART_CONTROLLER0)
    }

    pub fn uart_controller1() -> Self {
        Self::new(UART_CONTROLLER1)
    }

    pub fn uart_controller2() -> Self {
        Self::new(UART_CONTROLLER2)
    }

    pub fn initialize(&self, clock: u32, baud_rate: u32) {
        self.inner().init(clock, baud_rate);
    }

    pub fn read(&self) -> u8 {
        self.inner().read()
    }

    pub fn write(&self, buffer: &[u8]) {
        self.inner().write(buffer);
    }

    #[inline]
    fn new(base: usize) -> Self {
        Self {
            inner: base as *const S3C2440UartControllerInner,
        }
    }

    #[inline]
    fn inner(&self) -> &S3C2440UartControllerInner {
        unsafe { &(*self.inner) }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_calculate_baud_rate_divido() {
        // In the datasheet, when the clock is set to 40MHz, the dividor will be 21.
        assert_eq!(
            21,
            S3C2440UartControllerInner::calculate_baud_rate_dividor(40_000_000, 115200)
        );
    }
}
