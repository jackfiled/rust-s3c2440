use core::cell::UnsafeCell;
use core::ptr;

#[repr(transparent)]
pub(crate) struct Register(UnsafeCell<u32>);

impl Register {
    /// Read the register.
    #[inline]
    pub fn read(&self) -> u32 {
        unsafe { self.0.get().read_volatile() }
    }

    #[inline]
    pub fn read_u8(&self) -> u8 {
        unsafe { ptr::read_volatile(self.0.get() as *const u8) }
    }

    #[inline]
    pub fn read_u16(&self) -> u16 {
        unsafe { ptr::read_volatile(self.0.get() as *const u16) }
    }

    /// Write the register.
    #[inline]
    pub fn write(&self, value: u32) {
        unsafe {
            self.0.get().write_volatile(value);
        }
    }

    #[inline]
    pub fn write_u16(&self, value: u16) {
        unsafe {
            ptr::write_volatile(self.0.get() as *mut u16, value);
        }
    }

    #[inline]
    pub fn write_u8(&self, value: u8) {
        unsafe {
            ptr::write_volatile(self.0.get() as *mut u8, value);
        }
    }

    #[inline]
    pub fn set_bit(&self, value: u32, offset: u32) {
        let origin = self.read();

        // The 0 will be handled specially.
        if value == 0 {
            let mask = 1 << offset;
            self.write(origin & !mask);
            return;
        }

        self.write(origin | (value << offset));
    }

    #[inline]
    pub fn clear_bit(&self, offset: u32, width: u32) {
        let mask = ((1 << width) - 1) << offset;
        let origin = self.read();
        self.write(origin & !mask);
    }

    #[inline]
    pub fn is_bit_one(&self, offset: u32) -> bool {
        let result = self.read() & (1 << offset);
        result != 0
    }

    #[inline]
    pub fn address(&self) -> usize {
        self.0.get() as usize
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn reigster_read_write_test() {
        let register = Register(UnsafeCell::new(0));

        assert_eq!(0, register.read());
        register.write(2);
        assert_eq!(2, register.read());
    }

    #[test]
    pub fn register_set_bit_test() {
        let register = Register(UnsafeCell::new(0));

        assert_eq!(0, register.read());
        register.set_bit(1, 0);
        assert_eq!(1, register.read());
        register.set_bit(1, 1);
        assert_eq!(3, register.read());
    }

    #[test]
    pub fn register_set_zero_test() {
        let register = Register(UnsafeCell::new(0));

        assert_eq!(0, register.read());
        register.set_bit(1, 1);
        assert_eq!(2, register.read());
        register.set_bit(0, 1);
        assert_eq!(0, register.read());

        register.write(1);
        assert_eq!(1, register.read());
        register.set_bit(1, 1);
        register.set_bit(0, 1);
        assert_eq!(1, register.read());
    }
}
