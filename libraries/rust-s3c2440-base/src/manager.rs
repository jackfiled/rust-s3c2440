use crate::println;
use crate::system::PCLK;
use core::fmt::Arguments;
use rust_s3c2440_library::Global;
use rust_s3c2440_library::hardware::GPIOController;
use rust_s3c2440_library::hardware::S3C2440UartController;
use rust_s3c2440_library::software::{Log, LogLevel, S3C2440Console, set_logger};

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
    uart_controllers: [S3C2440UartController; 3],
    gpio_controller: GPIOController,
    console: Option<S3C2440Console>,
}

impl Manager {
    pub fn new() -> Self {
        Self {
            uart_controllers: [
                S3C2440UartController::uart_controller0(),
                S3C2440UartController::uart_controller1(),
                S3C2440UartController::uart_controller2(),
            ],
            gpio_controller: GPIOController::new(),
            console: None,
        }
    }

    pub fn initialize(&mut self, configuration: &InitializeConfiguration) {
        self.uart_controllers[configuration.uart_port]
            .initialize(PCLK, configuration.uart_buad_rate);

        self.gpio_controller.initialize();

        self.console = Some(S3C2440Console::new(
            self.uart_controllers[configuration.uart_port],
        ));

        set_logger(&Logger {}, configuration.log_level);
    }

    pub fn have_console(&self) -> Option<S3C2440Console> {
        self.console
    }
}

pub static MANAGER: Global<Manager> = Global::new();
