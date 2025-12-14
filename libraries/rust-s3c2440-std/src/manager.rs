use crate::io::initialize_console;
use crate::system::heap::initialize_heap;
use crate::system::interrupt::InterruptManager;
use core::cell::RefCell;
use log::{Level, LevelFilter, Metadata, Record, info};
use rust_s3c2440_hal::Global;

struct S3C2440Logger;

impl log::Log for S3C2440Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{}: {}", record.level(), record.args())
        }
    }

    fn flush(&self) {}
}

/// S3C2440 Board Manager.
pub struct Manager {
    interrupt_manager: RefCell<InterruptManager>,
}

impl Manager {
    pub fn initialize() {
        initialize_console();
        // Panic handler is usable.

        unsafe {
            log::set_logger_racy(&S3C2440Logger)
                .map(|()| log::set_max_level_racy(LevelFilter::Info))
                .unwrap()
        }

        // The print and log related macros should be usable.
        info!("Hello S3C2440!");

        initialize_heap();
        info!("Heap initialized.");

        let interrupt_manager = InterruptManager::new();

        MANAGER.init(Manager {
            interrupt_manager: RefCell::new(interrupt_manager),
        });

        // Enable interrupt handling before exiting.
        MANAGER
            .get()
            .unwrap()
            .interrupt()
            .borrow_mut()
            .enable_interrupt();
        info!("Interrupt has been enabled.");
    }

    pub fn interrupt(&self) -> &RefCell<InterruptManager> {
        &self.interrupt_manager
    }
}

pub static MANAGER: Global<Manager> = Global::new();
