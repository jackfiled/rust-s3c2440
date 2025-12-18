use core::cell::UnsafeCell;
use core::ops::{BitAndAssign, BitOrAssign};
use core::ptr;

#[repr(transparent)]
pub struct Register(UnsafeCell<u32>);

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
    pub fn set_bit(&self, value: u32, offset: u32, width: u32) {
        let origin = self.read();

        // Remove the target bits with a mask.
        let mask = ((1u32 << width) - 1) << offset;
        let mut result = origin & !mask;

        if value != 0 {
            result |= (value << offset) & mask;
        }

        self.write(result);
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

impl BitAndAssign<u32> for Register {
    fn bitand_assign(&mut self, rhs: u32) {
        self.write(self.read() & rhs);
    }
}

impl BitOrAssign<u32> for Register {
    fn bitor_assign(&mut self, rhs: u32) {
        self.write(self.read() | rhs);
    }
}

pub trait BitValue {
    fn value(&self) -> u32;
}

impl BitValue for bool {
    fn value(&self) -> u32 {
        match self {
            true => 1,
            false => 0,
        }
    }
}

pub fn empty_wrapper<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    f()
}

/// Macro to create a mutable reference to a statically allocated value
///
/// This macro returns a value with type `Option<&'static mut $ty>`. `Some($expr)` will be returned
/// the first time the macro is executed; further calls will return `None`. To avoid `unwrap`ping a
/// `None` variant the caller must ensure that the macro is called from a function that's executed
/// at most once in the whole lifetime of the program.
///
/// # Notes
/// This macro is unsound on multi cores systems.
///
/// For debuggability, you can set an explicit name for a singleton.  This name only shows up the
/// debugger and is not referencable from other code.  See example below.
///
/// # Example
///
/// ``` no_run
/// use cortex_m::singleton;
///
/// fn main() {
///     // OK if `main` is executed only once
///     let x: &'static mut bool = singleton!(: bool = false).unwrap();
///
///     let y = alias();
///     // BAD this second call to `alias` will definitively `panic!`
///     let y_alias = alias();
/// }
///
/// fn alias() -> &'static mut bool {
///     singleton!(: bool = false).unwrap()
/// }
///
/// fn singleton_with_name() {
///     // A name only for debugging purposes
///     singleton!(FOO_BUFFER: [u8; 1024] = [0u8; 1024]);
/// }
/// ```
#[macro_export]
macro_rules! singleton {
    ($name:ident: $ty:ty = $expr:expr) => {
        $crate::utils::empty_wrapper(|| {
            // this is a tuple of a MaybeUninit and a bool because using an Option here is
            // problematic:  Due to niche-optimization, an Option could end up producing a non-zero
            // initializer value which would move the entire static from `.bss` into `.data`.
            static mut $name: (::core::mem::MaybeUninit<$ty>, bool) = (::core::mem::MaybeUninit::uninit(), false);

            #[allow(unsafe_code)]
            let used = unsafe { $name.1 };
            if used {
                None
            } else {
                let expr = $expr;

                #[allow(unsafe_code)]
                unsafe {
                    $name.1 = true;
                    $name.0 = ::core::mem::MaybeUninit::new(expr);
                    Some(&mut *$name.0.as_mut_ptr())
                }
            }
        })

    };
    (: $ty:ty = $expr:expr) => {
        $crate::singleton!(VAR: $ty = $expr)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn register_read_write_test() {
        let register = Register(UnsafeCell::new(0));

        assert_eq!(0, register.read());
        register.write(2);
        assert_eq!(2, register.read());
    }

    #[test]
    pub fn register_set_bit_test() {
        let register = Register(UnsafeCell::new(0));

        assert_eq!(0, register.read());
        register.set_bit(1, 0, 1);
        assert_eq!(1, register.read());
        register.set_bit(1, 1, 1);
        assert_eq!(3, register.read());
    }

    #[test]
    pub fn register_set_zero_test() {
        let register = Register(UnsafeCell::new(0));

        assert_eq!(0, register.read());
        register.set_bit(1, 1, 1);
        assert_eq!(2, register.read());
        register.set_bit(0, 1, 1);
        assert_eq!(0, register.read());

        register.write(1);
        assert_eq!(1, register.read());
        register.set_bit(1, 1, 1);
        register.set_bit(0, 1, 1);
        assert_eq!(1, register.read());
    }

    #[test]
    pub fn uart_register_set_test() {
        let register = Register(UnsafeCell::new(0));

        assert_eq!(0, register.read());

        register.set_bit(2, 2 * 2, 2);
        register.set_bit(2, 2 * 3, 2);

        assert_eq!(0b10100000, register.read());
    }

    const L3C: u32 = 1 << 4;
    const L3D: u32 = 1 << 3;
    const L3M: u32 = 1 << 2;

    #[test]
    pub fn l3_register_set_test() {
        let register = Register(UnsafeCell::new(0));

        register.write(register.read() & !(L3D | L3M | L3C) | L3C);

        assert_eq!(16, register.read());
    }
}
