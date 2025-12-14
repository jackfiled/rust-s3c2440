use crate::system::PCLK;
use core::cell::UnsafeCell;
use core::fmt::{Display, Formatter, Write};
use core::mem::MaybeUninit;
use rust_s3c2440_hal::gpio::{PortHPin2, PortHPin3};
use rust_s3c2440_hal::nop;
use rust_s3c2440_hal::uart::{
    S3C2440UartController, S3C2440UartControllerBuilder, UartFifoOperation, UartOperation,
};

#[derive(Debug)]
pub struct S3C2440IoError {}

impl Display for S3C2440IoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "IO Error")
    }
}

type Result<T> = core::result::Result<T, S3C2440IoError>;

pub struct S3C2440Console<O: UartOperation> {
    pub uart_controller: S3C2440UartController<O>,
}

impl<O: UartOperation> S3C2440Console<O> {
    pub fn new(uart_controller: S3C2440UartController<O>) -> Self {
        Self { uart_controller }
    }

    pub fn put_char(&mut self, c: u8) -> usize {
        self.write_char(c as char).unwrap();
        0
    }

    pub fn get_char(&self) -> u8 {
        let mut result = 0u8;

        while self
            .uart_controller
            .try_read(core::slice::from_mut(&mut result))
            != 1
        {
            nop();
        }

        result
    }

    pub fn read(&self, buffer: &mut [u8]) -> Result<usize> {
        Ok(self.uart_controller.try_read(buffer))
    }
}

impl<O: UartOperation> Write for S3C2440Console<O> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let mut bytes = s.as_bytes();

        while !bytes.is_empty() {
            let c = self.uart_controller.try_write(bytes);
            bytes = &bytes[c..];
        }

        Ok(())
    }
}

pub struct GlobalConsole(UnsafeCell<MaybeUninit<S3C2440Console<UartFifoOperation<0>>>>);

unsafe impl Sync for GlobalConsole {}

impl GlobalConsole {
    const fn new() -> Self {
        Self(UnsafeCell::new(MaybeUninit::uninit()))
    }

    pub fn console(&self) -> &mut S3C2440Console<UartFifoOperation<0>> {
        unsafe { &mut (*((*self.0.get()).as_mut_ptr())) }
    }
}

pub static CONSOLE: GlobalConsole = GlobalConsole::new();

pub fn initialize_console() {
    let builder = S3C2440UartControllerBuilder::uart_controller0(
        PortHPin2::new().into_uart_transmit(),
        PortHPin3::new().into_uart_receive(),
    );

    let controller = builder.build_fifo::<PCLK, 115200>();

    unsafe {
        (*CONSOLE.0.get()).write(S3C2440Console::new(controller));
    }
}

pub fn get_char() -> u8 {
    CONSOLE.console().get_char()
}

#[allow(unused)]
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        use core::fmt::Write;
        let console = $crate::io::CONSOLE.console();
        console.write_fmt(core::format_args!($($arg)*)).unwrap();
    }
}

#[allow(unused)]
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\r\n"));
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let console = $crate::io::CONSOLE.console();
        console.write_fmt(core::format_args!($($arg)*)).unwrap();
        console.write_str("\r\n").unwrap();
    }}
}
