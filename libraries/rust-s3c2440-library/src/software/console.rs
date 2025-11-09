use crate::hardware::{S3C2440UartController, nop};
use core::fmt::Write;

#[derive(Copy, Clone)]
pub struct S3C2440Console {
    uart_controller: S3C2440UartController,
}

impl S3C2440Console {
    pub fn new(uart_controller: S3C2440UartController) -> Self {
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
}

impl Write for S3C2440Console {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let mut bytes = s.as_bytes();

        while !bytes.is_empty() {
            let c = self.uart_controller.try_write(bytes);
            bytes = &bytes[c..];
        }

        Ok(())
    }
}
