use core::fmt::{Display, Formatter};

#[repr(u32)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CpuMode {
    User = 0b10000,
    FastInterrupt = 0b10001,
    Interrupt = 0b10010,
    Management = 0b10011,
    Abort = 0b10111,
    Undefined = 0b11011,
    System = 0b11111,
}

impl CpuMode {
    pub const MASK: u32 = 0b11111;
}

impl From<u32> for CpuMode {
    fn from(value: u32) -> Self {
        match value {
            0b10000 => CpuMode::User,
            0b10001 => CpuMode::FastInterrupt,
            0b10010 => CpuMode::Interrupt,
            0b10011 => CpuMode::Management,
            0b10111 => CpuMode::Abort,
            0b11011 => CpuMode::Undefined,
            0b11111 => CpuMode::System,
            _ => panic!("{value} is not valid Mode value."),
        }
    }
}

impl Display for CpuMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            CpuMode::User => write!(f, "User"),
            CpuMode::FastInterrupt => write!(f, "FIQ"),
            CpuMode::Interrupt => write!(f, "IRQ"),
            CpuMode::Management => write!(f, "SVC"),
            CpuMode::Abort => write!(f, "abt"),
            CpuMode::System => write!(f, "sys"),
            CpuMode::Undefined => write!(f, "und"),
        }
    }
}

/// The base address of 3 UART controllers.
pub const UART_CONTROLLER_BASE: usize = 0x5000_0000;
/// The delta between UART controllers.
pub const UART_CONTROLLER_DELTA: usize = 0x4000;

pub const GPACON: usize = 0x56000000; // Port A control

pub const GPBCON: usize = 0x56000010; // Port B control

pub const GPCCON: usize = 0x56000020; // Port C control

pub const GPDCON: usize = 0x56000030; // Port D control

pub const GPECON: usize = 0x56000040; // Port E control

pub const GPFCON: usize = 0x56000050; // Port F control

pub const GPGCON: usize = 0x56000060; // Port G control

pub const GPHCON: usize = 0x56000070; // Port H control

pub const GPJCON: usize = 0x560000d0; // Port J control
