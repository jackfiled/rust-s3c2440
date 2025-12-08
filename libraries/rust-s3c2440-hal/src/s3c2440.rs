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
    pub const MASK: u32 = 0b00000;
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
