use crate::println;
use crate::support::console::S3C2440Console;
use crate::support::{Log, LogLevel, set_logger};
use crate::system::PCLK;
use core::cell::RefCell;
use core::fmt::Arguments;
use rust_s3c2440_hal::Global;
use rust_s3c2440_hal::gpio::{PortHPin2, PortHPin3, PortHPin4, PortHPin5, PortHPin6, PortHPin7};
use rust_s3c2440_hal::uart::{S3C2440UartController, S3C2440UartControllerBuilder};

pub struct InitializeConfiguration {
    pub uart_port: usize,
    pub uart_buad_rate: u32,
    pub log_level: LogLevel,
}

struct Logger {}

impl Log for Logger {
    fn log(&self, level: LogLevel, agrs: Arguments) {
        println!("{}: {}", level, agrs);
    }
}

/// S3C2440 Board Manager.
pub struct Manager {
    /// The UART controllers in S3C2440.
    uart_controller: S3C2440UartController,
    console: RefCell<S3C2440Console>,
}

impl Manager {
    pub fn initialize(configuration: InitializeConfiguration) {
        let uart_controller = match configuration.uart_port {
            0 => S3C2440UartControllerBuilder::uart_controller0(
                PortHPin2::init().into_uart_transmit(),
                PortHPin3::init().into_uart_receive(),
            )
            .initialize(PCLK, configuration.uart_buad_rate),
            1 => S3C2440UartControllerBuilder::uart_controller1(
                PortHPin4::init().into_uart_transmit(),
                PortHPin5::init().into_uart_receive(),
            )
            .initialize(PCLK, configuration.uart_buad_rate),
            2 => S3C2440UartControllerBuilder::uart_controller2(
                PortHPin6::init().into_uart_transmit(),
                PortHPin7::init().into_uart_receive(),
            )
            .initialize(PCLK, configuration.uart_buad_rate),
            _ => unreachable!(),
        };
        let console = S3C2440Console::new(uart_controller);

        MANAGER.init(Manager {
            uart_controller,
            console: RefCell::new(console),
        });

        set_logger(&Logger {}, configuration.log_level);
    }

    pub fn console(&self) -> &RefCell<S3C2440Console> {
        &self.console
    }
}

pub static MANAGER: Global<Manager> = Global::new();
