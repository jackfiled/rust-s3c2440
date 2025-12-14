#![no_std]
#![no_main]

mod app;

use rust_s3c2440_std::io::get_char;
use rust_s3c2440_std::{entry, println};

#[entry]
fn main() -> ! {
    println!("Reading test...");

    for _ in 0..100 {
        println!("Hello, TQ2440!");
    }

    loop {
        let c = get_char();
        match core::str::from_utf8(core::slice::from_ref(&c)) {
            Ok(s) => println!("Read char: {s}"),
            Err(e) => println!("Read error: {e}"),
        }
    }
}
