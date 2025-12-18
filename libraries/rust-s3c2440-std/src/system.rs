//! This file contains most S3C2440 constant definitions such as registers and configurations.
//! Some configurations are not very semantic and well-documented.
#![allow(dead_code)]

use crate::io::initialize_console;
use crate::system::heap::initialize_heap;
use crate::system::interrupt::InterruptManager;
use crate::utils::InterruptGuard;
use alloc::boxed::Box;
use core::arch::asm;
use core::cell::RefCell;
use core::fmt::{Display, Formatter};
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};
use log::{Level, LevelFilter, Metadata, Record, info};
use rust_s3c2440_hal::interrupt::InterruptSource;
use rust_s3c2440_hal::s3c2440::CpuMode;

pub mod clock;
pub(super) mod heap;
pub mod interrupt;
mod stack;
mod start;

/// Trigger a software interrupt(SWI).
/// This function must be inlined otherwise the returning address will be the `mov pc, lr`.
/// And under the interrupt handled the original lr register will be polluted when the software
/// interrupt is triggered under SVC mode as the handled mode is also SVC.
#[inline(always)]
pub fn software_interrupt() {
    unsafe {
        asm!("swi 114514");
    }
}

/// The CPSR(current program status register) abstraction.
#[repr(transparent)]
#[derive(Debug)]
pub struct StatusRegister(u32);

pub fn read_cpsr() -> StatusRegister {
    let result: u32;

    unsafe {
        asm!(
            "mrs {}, cpsr",
            out(reg) result,
        )
    }

    StatusRegister(result)
}

pub fn read_spsr() -> StatusRegister {
    let result: u32;

    unsafe {
        asm!(
        "mrs {}, spsr",
        out(reg) result,
        )
    }

    StatusRegister(result)
}

impl StatusRegister {
    pub fn cpu_mode(&self) -> CpuMode {
        CpuMode::from(self.0 & 0x1f)
    }

    pub fn write_cpsr(&self) {
        unsafe {
            asm!(
                "msr cpsr, {}",
                in(reg) self.0
            )
        }
    }

    #[inline]
    pub fn enable_interrupt(&mut self) {
        self.0 &= !(1 << 7);
    }

    #[inline]
    pub fn disable_interrupt(&mut self) {
        self.0 |= 1 << 7;
    }

    #[inline]
    pub fn interrupt_enabled(&self) -> bool {
        ((self.0 >> 7) & 1) == 0
    }

    #[inline]
    pub fn enable_fast_interrupt(&mut self) {
        self.0 &= !(1 << 6);
    }

    #[inline]
    pub fn disable_fast_interrupt(&mut self) {
        self.0 |= 1 << 6;
    }

    #[inline]
    pub fn fast_interrupt_enabled(&self) -> bool {
        ((self.0 >> 6) & 1) == 0
    }
}

impl Display for StatusRegister {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "CPU status: IRQ: {}, FIQ: {}, Mode: {}",
            if self.interrupt_enabled() {
                "Enabled"
            } else {
                "Disabled"
            },
            if self.fast_interrupt_enabled() {
                "Enabled"
            } else {
                "Disabled"
            },
            self.cpu_mode()
        )
    }
}

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

pub struct ManagerInner {
    interrupt_manager: RefCell<InterruptManager>,
}

/// S3C2440 Board Manager.
pub struct Manager {
    inner: MaybeUninit<ManagerInner>,
}

impl Deref for Manager {
    type Target = ManagerInner;

    fn deref(&self) -> &Self::Target {
        unsafe { &(*self.inner.as_ptr()) }
    }
}

impl DerefMut for Manager {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut (*self.inner.as_mut_ptr()) }
    }
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

        unsafe {
            MANAGER.inner.write(ManagerInner {
                interrupt_manager: RefCell::new(interrupt_manager),
            });
        }

        // Enable interrupt handling before exiting.
        get_manager().interrupt().borrow_mut().enable_interrupt();
        info!("Interrupt has been enabled.");
    }

    pub fn interrupt(&self) -> &RefCell<InterruptManager> {
        &self.interrupt_manager
    }
}

static mut MANAGER: Manager = Manager {
    inner: MaybeUninit::uninit(),
};

/// Get the board manager reference.
pub(crate) fn get_manager() -> &'static Manager {
    unsafe { &MANAGER }
}

pub fn register_interrupt(source: InterruptSource, handler: Box<dyn Fn()>) {
    InterruptGuard::with_disabled(|| {
        get_manager()
            .interrupt()
            .borrow_mut()
            .register_interrupt_handler(source, handler);
    });
}

pub fn unregister_interrupt(source: InterruptSource) {
    InterruptGuard::with_disabled(|| {
        get_manager()
            .interrupt()
            .borrow_mut()
            .unregister_interrupt_handler(source);
    });
}

pub fn register_software_interrupt(handler: Box<dyn Fn()>) {
    InterruptGuard::with_disabled(|| {
        get_manager()
            .interrupt()
            .borrow_mut()
            .register_software_interrupt_handler(handler);
    });
}

pub fn unregister_software_interrupt() {
    InterruptGuard::with_disabled(|| {
        get_manager()
            .interrupt()
            .borrow_mut()
            .unregister_software_interrupt_handler();
    });
}
