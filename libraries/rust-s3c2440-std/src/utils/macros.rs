#[allow(unused)]
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        use core::fmt::Write;
        let console = unsafe { $crate::MANAGER.get_unchecked().console() };
        console.borrow_mut().write_fmt(core::format_args!($($arg)*)).unwrap();
        $crate::MANAGER.get_unchecked().console().write_fmt(core::format_args!($($arg)*)).unwrap();
    }
}

#[allow(unused)]
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\r\n"));
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let console = unsafe { $crate::MANAGER.get_unchecked().console() };
        console.borrow_mut().write_fmt(core::format_args!($($arg)*)).unwrap();
        console.borrow_mut().write_str("\r\n").unwrap();
    }}
}
