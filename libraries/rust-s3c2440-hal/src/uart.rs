use crate::delay_cycles;
use crate::gpio::{
    PortHPin2, PortHPin3, PortHPin4, PortHPin5, PortHPin6, PortHPin7, UartReceive, UartTransmit,
};
use crate::s3c2440::{UART_CONTROLLER_BASE, UART_CONTROLLER_DELTA};
use crate::uart::sealed::Sealed;
use crate::utils::Register;
use bitflags::bitflags;
use core::error::Error;
use core::fmt::{Display, Formatter};
use core::ops::Deref;

mod sealed {
    pub trait Sealed {}
}

pub trait UartOperation: Sealed + Default {
    /// Read the UART port non-blockingly.
    /// # Returns
    /// The read byte count.
    fn try_read(&self, buffer: &mut [u8]) -> usize;

    /// Write the UART port non-blockingly.
    /// # Returns
    /// Written byte count.
    fn try_write(&self, buffer: &[u8]) -> usize;

    fn fifo_control_value() -> u32;
}

bitflags! {
    #[derive(Debug)]
    pub struct UartError: u32 {
        const OVERFLOW = 1 << 0;
        const VALIDATION = 1 << 1;
        const FRAME = 1 << 2;
        const BREAKPOINT = 1 << 3;
    }
}

impl Display for UartError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut first = true;

        if self.contains(UartError::OVERFLOW) {
            write!(f, "overflow")?;
            first = false;
        }
        if self.contains(UartError::VALIDATION) {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "validation error")?;
            first = false;
        }
        if self.contains(UartError::FRAME) {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "framing error")?;
            first = false;
        }
        if self.contains(UartError::BREAKPOINT) {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "break condition")?;
        }

        Ok(())
    }
}

impl Error for UartError {}

pub struct UartConfig {
    clock: u32,
    baud_rate: u32,
}

impl UartConfig {
    /// Calculate the baud rate divider value.
    /// Baud rate will be PCLK / ((divider + 1) * 16)
    const fn calculate_baud_rate_divider(&self) -> u32 {
        // divider = PCLK / (16 * baud_rate) - 1.
        // The value may be less than the standard value, so try to add 1 to find the nearest value.
        let divider1 = self.clock / (16 * self.baud_rate) - 1;
        let divider2 = divider1 + 1;

        let baud_rate1 = self.clock / ((divider1 + 1) * 16);
        let baud_rate2 = self.clock / ((divider2 + 1) * 16);

        if baud_rate1.abs_diff(self.baud_rate) < baud_rate2.abs_diff(self.baud_rate) {
            divider1
        } else {
            divider2
        }
    }
}

#[repr(C)]
pub struct S3C2440UartControllerInner {
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
    send_buffer: Register,
    /// URXH register.
    receive_buffer: Register,
    /// UBRDIV register.
    baud_rate_divisor: Register,
}

const fn uart_inner(n: usize) -> &'static S3C2440UartControllerInner {
    let inner =
        (UART_CONTROLLER_BASE + n * UART_CONTROLLER_DELTA) as *const S3C2440UartControllerInner;

    unsafe { &(*inner) }
}

impl S3C2440UartControllerInner {
    fn init(&self, divider_value: u32, fifo_control_value: u32) {
        // Line control register:  normal mode, no validation, 1 stop bit and length of 8 bits.
        self.line_control.write(0x3);

        // Control register: using PCLK, disabling all interrupts, no loopback, not sending break
        // signal and using poll mode.
        self.control.write(0x5);

        // FIFO configuration.
        self.fifo_control.write(fifo_control_value);

        // Modem: disable AFC.
        self.modem_control.write(0);

        // Set the baud rate divider.
        self.baud_rate_divisor.write(divider_value);

        // After setting values, wait for some time.
        delay_cycles(1000);
    }

    fn is_receive_buffer_ready(&self) -> bool {
        self.send_receive_status.is_bit_one(0)
    }

    fn is_sender_buffer_empty(&self) -> bool {
        self.send_receive_status.is_bit_one(1)
    }

    fn is_sender_ready(&self) -> bool {
        self.send_receive_status.is_bit_one(2)
    }

    fn is_receive_fifo_full(&self) -> bool {
        self.fifo_status.is_bit_one(6)
    }

    fn is_send_fifo_full(&self) -> bool {
        self.fifo_status.is_bit_one(1)
    }

    fn read_error(&self) -> UartError {
        let mut error = UartError::empty();
        let value = self.receive_error_status.read();

        for i in 0..3 {
            if ((value >> i) & 0x1) == 0x1 {
                error |= UartError::from_bits(i).unwrap();
            }
        }

        error
    }

    /// Fifo status: (sending count, receiving count)
    fn read_fifo_status(&self) -> (u32, u32) {
        let value = self.fifo_status.read();

        ((value >> 8) & 0x3f, value & 0x3f)
    }
}

pub struct UartNonFifoOperation<const N: usize> {}

impl<const N: usize> Deref for UartNonFifoOperation<N> {
    type Target = S3C2440UartControllerInner;

    fn deref(&self) -> &Self::Target {
        uart_inner(N)
    }
}

impl<const N: usize> Sealed for UartNonFifoOperation<N> {}

impl<const N: usize> Default for UartNonFifoOperation<N> {
    fn default() -> Self {
        UartNonFifoOperation {}
    }
}

impl<const N: usize> UartOperation for UartNonFifoOperation<N> {
    fn try_read(&self, buffer: &mut [u8]) -> usize {
        let mut count = 0;
        for i in 0..buffer.len() {
            if !self.is_receive_buffer_ready() {
                break;
            }

            buffer[i] = self.receive_buffer.read() as u8;
            // There are error FIFO, so always reading the error status register.
            count += 1;
        }

        count
    }

    fn try_write(&self, buffer: &[u8]) -> usize {
        let mut count = 0;
        for &b in buffer {
            if self.is_sender_buffer_empty() {
                self.send_buffer.write_u8(b);
                count += 1;
            } else {
                break;
            }
        }

        count
    }

    fn fifo_control_value() -> u32 {
        0
    }
}

pub struct UartFifoOperation<const N: usize> {}

impl<const N: usize> Deref for UartFifoOperation<N> {
    type Target = S3C2440UartControllerInner;

    fn deref(&self) -> &Self::Target {
        uart_inner(N)
    }
}

impl<const N: usize> Sealed for UartFifoOperation<N> {}

impl<const N: usize> Default for UartFifoOperation<N> {
    fn default() -> Self {
        UartFifoOperation {}
    }
}

impl<const N: usize> UartOperation for UartFifoOperation<N> {
    fn try_read(&self, buffer: &mut [u8]) -> usize {
        let count = buffer.len().min(self.read_fifo_status().1 as usize);

        for v in buffer.iter_mut().take(count) {
            *v = self.receive_buffer.read() as u8;
            // There are error FIFO as data buffer, so always reading FIFO.
            _ = self.read_error();
        }

        count
    }

    fn try_write(&self, buffer: &[u8]) -> usize {
        // Then fifo status is the length of buffer, so the empty length is 64 - it.
        let count = buffer.len().min(64 - self.read_fifo_status().0 as usize);

        for &v in buffer.iter().take(count) {
            self.send_buffer.write_u8(v)
        }

        count
    }

    fn fifo_control_value() -> u32 {
        0b111
    }
}

pub struct S3C2440UartController<O: UartOperation> {
    operation: O,
}

impl<O: UartOperation> S3C2440UartController<O> {
    #[inline(always)]
    pub fn try_read(&self, buffer: &mut [u8]) -> usize {
        self.operation.try_read(buffer)
    }

    #[inline(always)]
    pub fn try_write(&self, buffer: &[u8]) -> usize {
        self.operation.try_write(buffer)
    }
}

pub struct S3C2440UartControllerBuilder<const N: usize> {}

impl S3C2440UartControllerBuilder<0> {
    pub fn uart_controller0(
        _: PortHPin2<UartTransmit>,
        _: PortHPin3<UartReceive>,
    ) -> S3C2440UartControllerBuilder<0> {
        S3C2440UartControllerBuilder {}
    }

    pub fn uart_controller1(
        _: PortHPin4<UartTransmit>,
        _: PortHPin5<UartReceive>,
    ) -> S3C2440UartControllerBuilder<1> {
        S3C2440UartControllerBuilder {}
    }

    pub fn uart_controller2(
        _: PortHPin6<UartTransmit>,
        _: PortHPin7<UartReceive>,
    ) -> S3C2440UartControllerBuilder<2> {
        S3C2440UartControllerBuilder {}
    }
}

impl<const N: usize> S3C2440UartControllerBuilder<N> {
    pub fn build<O: UartOperation, const CLK: u32, const BAUD: u32>(
        self,
    ) -> S3C2440UartController<O> {
        uart_inner(N).init(
            Self::calculate_baud_rate_divider(CLK, BAUD),
            O::fifo_control_value(),
        );

        S3C2440UartController {
            operation: O::default(),
        }
    }

    pub fn build_fifo<const CLK: u32, const BAUD: u32>(
        self,
    ) -> S3C2440UartController<UartFifoOperation<N>> {
        uart_inner(N).init(
            Self::calculate_baud_rate_divider(CLK, BAUD),
            UartFifoOperation::<0>::fifo_control_value(),
        );

        S3C2440UartController {
            operation: UartFifoOperation::default(),
        }
    }

    /// Calculate the baud rate divider value.
    /// Baud rate will be PCLK / ((divider + 1) * 16)
    const fn calculate_baud_rate_divider(clock: u32, baud_rate: u32) -> u32 {
        // divider = PCLK / (16 * baud_rate) - 1.
        // The value may be less than the standard value, so try to add 1 to find the nearest value.
        let divider1 = clock / (16 * baud_rate) - 1;
        let divider2 = divider1 + 1;

        let baud_rate1 = clock / ((divider1 + 1) * 16);
        let baud_rate2 = clock / ((divider2 + 1) * 16);

        if baud_rate1.abs_diff(baud_rate) < baud_rate2.abs_diff(baud_rate) {
            divider1
        } else {
            divider2
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_calculate_baud_rate_divider() {
        // In the datasheet, when the clock is set to 40MHz, the divider will be 21.
        assert_eq!(
            21,
            S3C2440UartControllerBuilder::<0>::calculate_baud_rate_divider(40_000_000, 115200)
        );
        assert_eq!(
            27,
            S3C2440UartControllerBuilder::<0>::calculate_baud_rate_divider(52_500_000, 115200)
        );
    }
}
