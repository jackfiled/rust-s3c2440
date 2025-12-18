use crate::s3c2440::INTERRUPT_CONTROLLER;
use crate::singleton;
use crate::utils::Register;
use core::fmt;
use core::ops::Deref;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[repr(u32)]
enum InterruptSourceInner {
    External0 = 0,
    External1,
    External2,
    External3,
    External4_7,
    External8_23,
    Camera,
    BatteryFault,
    Tick,
    Watchdog,
    Timer0,
    Timer1,
    Timer2,
    Timer3,
    Timer4,
    Uart2,
    Lcd,
    Dma0,
    Dma1,
    Dma2,
    Dma3,
    Sdi,
    Spi0,
    Uart1,
    NandFlash,
    UsbDevice,
    UsbHost,
    Iic,
    Uart0,
    Spi1,
    Rtc,
    Adc,
}

impl From<u32> for InterruptSourceInner {
    fn from(value: u32) -> Self {
        match value {
            0 => InterruptSourceInner::External0,
            1 => InterruptSourceInner::External1,
            2 => InterruptSourceInner::External2,
            3 => InterruptSourceInner::External3,
            4 => InterruptSourceInner::External4_7,
            5 => InterruptSourceInner::External8_23,
            6 => InterruptSourceInner::Camera,
            7 => InterruptSourceInner::BatteryFault,
            8 => InterruptSourceInner::Tick,
            9 => InterruptSourceInner::Watchdog,
            10 => InterruptSourceInner::Timer0,
            11 => InterruptSourceInner::Timer1,
            12 => InterruptSourceInner::Timer2,
            13 => InterruptSourceInner::Timer3,
            14 => InterruptSourceInner::Timer4,
            15 => InterruptSourceInner::Uart2,
            16 => InterruptSourceInner::Lcd,
            17 => InterruptSourceInner::Dma0,
            18 => InterruptSourceInner::Dma1,
            19 => InterruptSourceInner::Dma2,
            20 => InterruptSourceInner::Dma3,
            21 => InterruptSourceInner::Sdi,
            22 => InterruptSourceInner::Spi0,
            23 => InterruptSourceInner::Uart1,
            24 => InterruptSourceInner::NandFlash,
            25 => InterruptSourceInner::UsbDevice,
            26 => InterruptSourceInner::UsbHost,
            27 => InterruptSourceInner::Iic,
            28 => InterruptSourceInner::Uart0,
            29 => InterruptSourceInner::Spi1,
            30 => InterruptSourceInner::Rtc,
            31 => InterruptSourceInner::Adc,
            _ => panic!("Invalid InterruptSource value: {}", value),
        }
    }
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
enum InterruptSubSourceInner {
    Uart0Receive = 0,
    Uart0Send,
    Uart0Error,
    Uart1Receive,
    Uart1Send,
    Uart1Error,
    Uart2Receive,
    Uart2Send,
    Uart2Error,
    Touch,
    Adc,
    CameraC,
    CameraP,
    Watchdog,
    Ac97,
}

impl From<u32> for InterruptSubSourceInner {
    fn from(value: u32) -> Self {
        match value {
            0 => InterruptSubSourceInner::Uart0Receive,
            1 => InterruptSubSourceInner::Uart0Send,
            2 => InterruptSubSourceInner::Uart0Error,
            3 => InterruptSubSourceInner::Uart1Receive,
            4 => InterruptSubSourceInner::Uart1Send,
            5 => InterruptSubSourceInner::Uart1Error,
            6 => InterruptSubSourceInner::Uart2Receive,
            7 => InterruptSubSourceInner::Uart2Send,
            8 => InterruptSubSourceInner::Uart2Error,
            9 => InterruptSubSourceInner::Touch,
            10 => InterruptSubSourceInner::Adc,
            11 => InterruptSubSourceInner::CameraC,
            12 => InterruptSubSourceInner::CameraP,
            13 => InterruptSubSourceInner::Watchdog,
            14 => InterruptSubSourceInner::Ac97,
            _ => panic!("Invalid InterruptSubSource value: {}", value),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum UartInterruptSource {
    Receive,
    Send,
    Error,
}

impl From<UartInterruptSource> for usize {
    fn from(value: UartInterruptSource) -> Self {
        match value {
            UartInterruptSource::Receive => 0,
            UartInterruptSource::Send => 1,
            UartInterruptSource::Error => 2,
        }
    }
}

impl fmt::Display for UartInterruptSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UartInterruptSource::Receive => write!(f, "Receive"),
            UartInterruptSource::Send => write!(f, "Send"),
            UartInterruptSource::Error => write!(f, "Error"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum CameraInterruptSource {
    Preview,
    Codec,
}

#[derive(Debug, Copy, Clone)]
pub enum AdcInterruptSource {
    Adc,
    Touch,
}

#[derive(Debug, Copy, Clone)]
pub enum WatchdogInterruptSource {
    Watchdog,
    Ac97,
}

/// The 60 interrupt sources supported by S3C2440.
/// The LCD interrupt source contains INT_FrSyn and INT_FiCnt, but no sub source support like
/// external4_7 and external8_23.
/// So there are only 60 - 18 - 1 = 41 interrupt sources.
#[derive(Debug, Copy, Clone)]
pub enum InterruptSource {
    External0,
    External1,
    External2,
    External3,
    External4_7,
    External8_23,
    Camera(CameraInterruptSource),
    BatteryFault,
    Tick,
    Watchdog(WatchdogInterruptSource),
    Timer0,
    Timer1,
    Timer2,
    Timer3,
    Timer4,
    Uart2(UartInterruptSource),
    Lcd,
    Dma0,
    Dma1,
    Dma2,
    Dma3,
    Sdi,
    Spi0,
    Uart1(UartInterruptSource),
    NandFlash,
    UsbDevice,
    UsbHost,
    Iic,
    Uart0(UartInterruptSource),
    Spi1,
    Rtc,
    Adc(AdcInterruptSource),
}

impl InterruptSource {
    pub const INTERRUPT_SOURCE_COUNT: usize = 41;

    fn uart_source_map(
        source: &UartInterruptSource,
    ) -> (InterruptSourceInner, Option<InterruptSubSourceInner>) {
        match source {
            UartInterruptSource::Receive => (
                InterruptSourceInner::Uart2,
                Some(InterruptSubSourceInner::Uart2Receive),
            ),
            UartInterruptSource::Send => (
                InterruptSourceInner::Uart2,
                Some(InterruptSubSourceInner::Uart2Send),
            ),
            UartInterruptSource::Error => (
                InterruptSourceInner::Uart2,
                Some(InterruptSubSourceInner::Uart2Error),
            ),
        }
    }

    fn inner_map(&self) -> (InterruptSourceInner, Option<InterruptSubSourceInner>) {
        match self {
            InterruptSource::External0 => (InterruptSourceInner::External0, None),
            InterruptSource::External1 => (InterruptSourceInner::External1, None),
            InterruptSource::External2 => (InterruptSourceInner::External2, None),
            InterruptSource::External3 => (InterruptSourceInner::External3, None),
            InterruptSource::External4_7 => (InterruptSourceInner::External4_7, None),
            InterruptSource::External8_23 => (InterruptSourceInner::External8_23, None),
            InterruptSource::Camera(c) => match c {
                CameraInterruptSource::Codec => (
                    InterruptSourceInner::Camera,
                    Some(InterruptSubSourceInner::CameraC),
                ),
                CameraInterruptSource::Preview => (
                    InterruptSourceInner::Camera,
                    Some(InterruptSubSourceInner::CameraP),
                ),
            },
            InterruptSource::BatteryFault => (InterruptSourceInner::BatteryFault, None),
            InterruptSource::Tick => (InterruptSourceInner::Tick, None),
            InterruptSource::Watchdog(w) => match w {
                WatchdogInterruptSource::Watchdog => (
                    InterruptSourceInner::Watchdog,
                    Some(InterruptSubSourceInner::Watchdog),
                ),
                WatchdogInterruptSource::Ac97 => (
                    InterruptSourceInner::Watchdog,
                    Some(InterruptSubSourceInner::Ac97),
                ),
            },
            InterruptSource::Timer0 => (InterruptSourceInner::Timer0, None),
            InterruptSource::Timer1 => (InterruptSourceInner::Timer1, None),
            InterruptSource::Timer2 => (InterruptSourceInner::Timer2, None),
            InterruptSource::Timer3 => (InterruptSourceInner::Timer3, None),
            InterruptSource::Timer4 => (InterruptSourceInner::Timer4, None),
            InterruptSource::Uart2(u) => Self::uart_source_map(u),
            InterruptSource::Lcd => (InterruptSourceInner::Lcd, None),
            InterruptSource::Dma0 => (InterruptSourceInner::Dma0, None),
            InterruptSource::Dma1 => (InterruptSourceInner::Dma1, None),
            InterruptSource::Dma2 => (InterruptSourceInner::Dma2, None),
            InterruptSource::Dma3 => (InterruptSourceInner::Dma3, None),
            InterruptSource::Sdi => (InterruptSourceInner::Sdi, None),
            InterruptSource::Spi0 => (InterruptSourceInner::Spi0, None),
            InterruptSource::Uart1(u) => Self::uart_source_map(u),
            InterruptSource::NandFlash => (InterruptSourceInner::NandFlash, None),
            InterruptSource::UsbDevice => (InterruptSourceInner::UsbDevice, None),
            InterruptSource::UsbHost => (InterruptSourceInner::UsbHost, None),
            InterruptSource::Iic => (InterruptSourceInner::Iic, None),
            InterruptSource::Uart0(u) => Self::uart_source_map(u),
            InterruptSource::Spi1 => (InterruptSourceInner::Spi1, None),
            InterruptSource::Rtc => (InterruptSourceInner::Rtc, None),
            InterruptSource::Adc(a) => match a {
                AdcInterruptSource::Adc => (
                    InterruptSourceInner::Adc,
                    Some(InterruptSubSourceInner::Adc),
                ),
                AdcInterruptSource::Touch => (
                    InterruptSourceInner::Adc,
                    Some(InterruptSubSourceInner::Touch),
                ),
            },
        }
    }
}

impl From<InterruptSource> for usize {
    fn from(source: InterruptSource) -> usize {
        match source {
            InterruptSource::External0 => 0,
            InterruptSource::External1 => 1,
            InterruptSource::External2 => 2,
            InterruptSource::External3 => 3,
            InterruptSource::External4_7 => 4,
            InterruptSource::External8_23 => 5,
            InterruptSource::Camera(camera_source) => match camera_source {
                CameraInterruptSource::Preview => 6,
                CameraInterruptSource::Codec => 7,
            },
            InterruptSource::BatteryFault => 8,
            InterruptSource::Tick => 9,
            InterruptSource::Watchdog(watchdog_source) => match watchdog_source {
                WatchdogInterruptSource::Watchdog => 10,
                WatchdogInterruptSource::Ac97 => 11,
            },
            InterruptSource::Timer0 => 12,
            InterruptSource::Timer1 => 13,
            InterruptSource::Timer2 => 14,
            InterruptSource::Timer3 => 15,
            InterruptSource::Timer4 => 16,
            InterruptSource::Uart2(uart_source) => 17 + usize::from(uart_source),
            InterruptSource::Lcd => 20,
            InterruptSource::Dma0 => 21,
            InterruptSource::Dma1 => 22,
            InterruptSource::Dma2 => 23,
            InterruptSource::Dma3 => 24,
            InterruptSource::Sdi => 25,
            InterruptSource::Spi0 => 26,
            InterruptSource::Uart1(uart_source) => 27 + usize::from(uart_source),
            InterruptSource::NandFlash => 30,
            InterruptSource::UsbDevice => 31,
            InterruptSource::UsbHost => 32,
            InterruptSource::Iic => 33,
            InterruptSource::Uart0(uart_source) => 34 + usize::from(uart_source),
            InterruptSource::Spi1 => 37,
            InterruptSource::Rtc => 38,
            InterruptSource::Adc(adc_source) => match adc_source {
                AdcInterruptSource::Touch => 39,
                AdcInterruptSource::Adc => 40,
            },
        }
    }
}

impl fmt::Display for InterruptSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InterruptSource::External0 => write!(f, "External0"),
            InterruptSource::External1 => write!(f, "External1"),
            InterruptSource::External2 => write!(f, "External2"),
            InterruptSource::External3 => write!(f, "External3"),
            InterruptSource::External4_7 => write!(f, "External4_7"),
            InterruptSource::External8_23 => write!(f, "External8_23"),
            InterruptSource::Camera(source) => match source {
                CameraInterruptSource::Codec => write!(f, "CameraC"),
                CameraInterruptSource::Preview => write!(f, "CameraP"),
            },
            InterruptSource::BatteryFault => write!(f, "BatteryFault"),
            InterruptSource::Tick => write!(f, "Tick"),
            InterruptSource::Watchdog(source) => match source {
                WatchdogInterruptSource::Ac97 => write!(f, "AC97"),
                WatchdogInterruptSource::Watchdog => write!(f, "Watchdog"),
            },
            InterruptSource::Timer0 => write!(f, "Timer0"),
            InterruptSource::Timer1 => write!(f, "Timer1"),
            InterruptSource::Timer2 => write!(f, "Timer2"),
            InterruptSource::Timer3 => write!(f, "Timer3"),
            InterruptSource::Timer4 => write!(f, "Timer4"),
            InterruptSource::Uart2(source) => write!(f, "Uart2({})", source),
            InterruptSource::Lcd => write!(f, "LCD"),
            InterruptSource::Dma0 => write!(f, "DMA0"),
            InterruptSource::Dma1 => write!(f, "DMA1"),
            InterruptSource::Dma2 => write!(f, "DMA2"),
            InterruptSource::Dma3 => write!(f, "DMA3"),
            InterruptSource::Sdi => write!(f, "SDI"),
            InterruptSource::Spi0 => write!(f, "SPI0"),
            InterruptSource::Uart1(source) => write!(f, "Uart1({})", source),
            InterruptSource::NandFlash => write!(f, "NAND Flash"),
            InterruptSource::UsbDevice => write!(f, "USB Device"),
            InterruptSource::UsbHost => write!(f, "USB Host"),
            InterruptSource::Iic => write!(f, "IIC"),
            InterruptSource::Uart0(source) => write!(f, "Uart0({})", source),
            InterruptSource::Spi1 => write!(f, "SPI1"),
            InterruptSource::Rtc => write!(f, "RTC"),
            InterruptSource::Adc(source) => match source {
                AdcInterruptSource::Adc => write!(f, "ADC"),
                AdcInterruptSource::Touch => write!(f, "Touch"),
            },
        }
    }
}

#[repr(C)]
pub struct InterruptControllerInner {
    source_pending: Register,
    interrupt_mode: Register,
    interrupt_mask: Register,
    priority: Register,
    interrupt_pending: Register,
    interrupt_offset: Register,
    sub_source_pending: Register,
    sub_interrupt_mask: Register,
}

pub struct InterruptController {
    inner: *const InterruptControllerInner,
}

impl Deref for InterruptController {
    type Target = InterruptControllerInner;

    fn deref(&self) -> &Self::Target {
        unsafe { &(*self.inner) }
    }
}

impl InterruptController {
    fn new() -> Self {
        let controller = Self {
            inner: INTERRUPT_CONTROLLER as *const InterruptControllerInner,
        };

        // When initializing, disable all interrupts.
        controller.interrupt_mask.write(u32::MAX);
        controller.sub_interrupt_mask.write(u32::MAX);

        // Set the interrupt in IRQ mode.
        controller.interrupt_mode.write(0);

        controller
    }

    pub fn enable_interrupt(&mut self, source: InterruptSource) {
        let (s1, s2) = source.inner_map();
        self.interrupt_mask.set_bit(0, s1 as u32, 1);
        if let Some(s) = s2 {
            self.sub_interrupt_mask.set_bit(0, s as u32, 1);
        }
    }

    pub fn disable_interrupt(&mut self, source: InterruptSource) {
        let (s1, s2) = source.inner_map();
        if let Some(s) = s2 {
            self.sub_interrupt_mask.set_bit(1, s as u32, 1);
        }
        self.interrupt_mode.set_bit(1, s1 as u32, 1);
    }

    pub fn is_requesting(&self, source: InterruptSource) -> bool {
        let (s1, s2) = source.inner_map();

        self.source_pending.is_bit_one(s1 as u32)
            && s2.is_none_or(|s| self.sub_source_pending.is_bit_one(s as u32))
    }

    pub fn is_handling(&self, source: InterruptSource) -> bool {
        let (s1, _) = source.inner_map();

        self.interrupt_pending.is_bit_one(s1 as u32)
    }

    pub fn read_handling(&self) -> InterruptSource {
        let s1: InterruptSourceInner = self.interrupt_offset.read().into();

        match s1 {
            InterruptSourceInner::External0 => InterruptSource::External0,
            InterruptSourceInner::External1 => InterruptSource::External1,
            InterruptSourceInner::External2 => InterruptSource::External2,
            InterruptSourceInner::External3 => InterruptSource::External3,
            InterruptSourceInner::External4_7 => InterruptSource::External4_7,
            InterruptSourceInner::External8_23 => InterruptSource::External8_23,
            InterruptSourceInner::Camera => {
                if self
                    .sub_source_pending
                    .is_bit_one(InterruptSubSourceInner::CameraP as u32)
                {
                    InterruptSource::Camera(CameraInterruptSource::Preview)
                } else if self
                    .sub_source_pending
                    .is_bit_one(InterruptSubSourceInner::CameraC as u32)
                {
                    InterruptSource::Camera(CameraInterruptSource::Codec)
                } else {
                    unreachable!()
                }
            }
            InterruptSourceInner::BatteryFault => InterruptSource::BatteryFault,
            InterruptSourceInner::Tick => InterruptSource::Tick,
            InterruptSourceInner::Watchdog => {
                if self
                    .sub_source_pending
                    .is_bit_one(InterruptSubSourceInner::Watchdog as u32)
                {
                    InterruptSource::Watchdog(WatchdogInterruptSource::Watchdog)
                } else if self
                    .sub_source_pending
                    .is_bit_one(InterruptSubSourceInner::Ac97 as u32)
                {
                    InterruptSource::Watchdog(WatchdogInterruptSource::Ac97)
                } else {
                    unreachable!()
                }
            }
            InterruptSourceInner::Timer0 => InterruptSource::Timer0,
            InterruptSourceInner::Timer1 => InterruptSource::Timer1,
            InterruptSourceInner::Timer2 => InterruptSource::Timer2,
            InterruptSourceInner::Timer3 => InterruptSource::Timer3,
            InterruptSourceInner::Timer4 => InterruptSource::Timer4,
            InterruptSourceInner::Uart2 => {
                if self
                    .sub_source_pending
                    .is_bit_one(InterruptSubSourceInner::Uart2Send as u32)
                {
                    InterruptSource::Uart2(UartInterruptSource::Send)
                } else if self
                    .sub_source_pending
                    .is_bit_one(InterruptSubSourceInner::Uart2Receive as u32)
                {
                    InterruptSource::Uart2(UartInterruptSource::Receive)
                } else if self
                    .sub_source_pending
                    .is_bit_one(InterruptSubSourceInner::Uart2Error as u32)
                {
                    InterruptSource::Uart2(UartInterruptSource::Error)
                } else {
                    unreachable!()
                }
            }
            InterruptSourceInner::Lcd => InterruptSource::Lcd,
            InterruptSourceInner::Dma0 => InterruptSource::Dma0,
            InterruptSourceInner::Dma1 => InterruptSource::Dma1,
            InterruptSourceInner::Dma2 => InterruptSource::Dma2,
            InterruptSourceInner::Dma3 => InterruptSource::Dma3,
            InterruptSourceInner::Sdi => InterruptSource::Sdi,
            InterruptSourceInner::Spi0 => InterruptSource::Spi0,
            InterruptSourceInner::Uart1 => {
                if self
                    .sub_source_pending
                    .is_bit_one(InterruptSubSourceInner::Uart1Send as u32)
                {
                    InterruptSource::Uart1(UartInterruptSource::Send)
                } else if self
                    .sub_source_pending
                    .is_bit_one(InterruptSubSourceInner::Uart1Receive as u32)
                {
                    InterruptSource::Uart1(UartInterruptSource::Receive)
                } else if self
                    .sub_source_pending
                    .is_bit_one(InterruptSubSourceInner::Uart1Error as u32)
                {
                    InterruptSource::Uart1(UartInterruptSource::Error)
                } else {
                    unreachable!()
                }
            }
            InterruptSourceInner::NandFlash => InterruptSource::NandFlash,
            InterruptSourceInner::UsbDevice => InterruptSource::UsbDevice,
            InterruptSourceInner::UsbHost => InterruptSource::UsbHost,
            InterruptSourceInner::Iic => InterruptSource::Iic,
            InterruptSourceInner::Uart0 => {
                if self
                    .sub_source_pending
                    .is_bit_one(InterruptSubSourceInner::Uart0Send as u32)
                {
                    InterruptSource::Uart0(UartInterruptSource::Send)
                } else if self
                    .sub_source_pending
                    .is_bit_one(InterruptSubSourceInner::Uart0Receive as u32)
                {
                    InterruptSource::Uart0(UartInterruptSource::Receive)
                } else if self
                    .sub_source_pending
                    .is_bit_one(InterruptSubSourceInner::Uart0Error as u32)
                {
                    InterruptSource::Uart0(UartInterruptSource::Error)
                } else {
                    unreachable!()
                }
            }
            InterruptSourceInner::Spi1 => InterruptSource::Spi1,
            InterruptSourceInner::Rtc => InterruptSource::Rtc,
            InterruptSourceInner::Adc => {
                if self
                    .sub_source_pending
                    .is_bit_one(InterruptSubSourceInner::Adc as u32)
                {
                    InterruptSource::Adc(AdcInterruptSource::Adc)
                } else if self
                    .sub_source_pending
                    .is_bit_one(InterruptSubSourceInner::Touch as u32)
                {
                    InterruptSource::Adc(AdcInterruptSource::Touch)
                } else {
                    unreachable!()
                }
            }
        }
    }

    pub fn clear_pending_interrupt(&mut self, source: InterruptSource) {
        let (s1, s2) = source.inner_map();

        // First clear the source pending register.
        self.source_pending.write(1 << s1 as u32);
        if let Some(s) = s2 {
            self.sub_source_pending.write(1 << s as u32);
        }
        // Then clear the interrupt pending register.
        self.interrupt_pending.write(1 << s1 as u32);
    }
}

pub fn get_interrupt_controller() -> &'static mut InterruptController {
    singleton!(: InterruptController = InterruptController::new()).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interrupt_mode_test() {
        assert_eq!(31, InterruptSourceInner::Adc as u32);

        assert_eq!(0, InterruptSubSourceInner::Uart0Receive as u32);
        assert_eq!(14, InterruptSubSourceInner::Ac97 as u32);
    }
}
