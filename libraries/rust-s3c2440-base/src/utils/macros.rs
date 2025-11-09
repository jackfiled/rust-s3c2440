#[allow(unused)]
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        use core::fmt::Write;
        if let Some(mut console) = unsafe {$crate::MANAGER.get_unchecked().have_console()} {
            console.write_fmt(core::format_args!($($arg)*)).unwrap();
        }
    }
}

#[allow(unused)]
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\r\n"));
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        if let Some(mut console) = unsafe {$crate::MANAGER.get_unchecked().have_console()} {
            console.write_fmt(core::format_args!($($arg)*)).unwrap();
            console.write_str("\r\n").unwrap();
        }
    }}
}
