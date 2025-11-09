#![no_std]
#![allow(dead_code)]
use core::cell::UnsafeCell;
use core::ptr;

pub mod hardware;
pub mod software;

#[repr(transparent)]
pub struct Register(UnsafeCell<u32>);

impl Register {
    /// Read the register.
    pub fn read(&self) -> u32 {
        unsafe { self.0.get().read_volatile() }
    }

    /// Write the register.
    pub fn write(&self, value: u32) {
        unsafe {
            self.0.get().write_volatile(value);
        }
    }
}

/// A global variable container with initialize-once guaranteed.
pub struct Global<T> {
    initialized: UnsafeCell<bool>,
    value: UnsafeCell<core::mem::MaybeUninit<T>>,
}

// Only safety under single-core system and should be initialized under interrupt disabled context.
unsafe impl<T> Sync for Global<T> {}

impl<T> Global<T> {
    pub const fn new() -> Self {
        Self {
            initialized: UnsafeCell::new(false),
            value: UnsafeCell::new(core::mem::MaybeUninit::uninit()),
        }
    }

    pub fn init(&self, val: T) {
        unsafe {
            let init_ptr = self.initialized.get();
            if !(*init_ptr) {
                ptr::write(self.value.get() as *mut T, val);
                *init_ptr = true;
            } else {
                if cfg!(debug_assertions) {
                    panic!("Global::init called twice!");
                }
            }
        }
    }

    pub fn get(&self) -> Option<&T> {
        unsafe {
            if *self.initialized.get() {
                Some(&*(self.value.get() as *const T))
            } else {
                None
            }
        }
    }

    pub unsafe fn get_unchecked(&self) -> &T {
        unsafe { &*(self.value.get() as *const T) }
    }
}
