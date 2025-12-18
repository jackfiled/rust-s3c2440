#![no_std]
#![no_main]
extern crate alloc;

use alloc::boxed::Box;
use alloc::rc::Rc;
use core::cell::RefCell;
use rust_s3c2440_std::system::clock::delay_ms;
use rust_s3c2440_std::system::{
    register_software_interrupt, software_interrupt, unregister_software_interrupt,
};
use rust_s3c2440_std::utils::debug::print_debug_info;
use rust_s3c2440_std::{entry, println};

#[entry]
fn main() -> ! {
    let count = Rc::new(RefCell::new(0));
    let count2 = count.clone();
    register_software_interrupt(Box::new(move || {
        *count.borrow_mut() += 1;
        println!("count += 1");
    }));

    print_debug_info();
    println!("Enabled interrupt.");

    println!("Try to trigger software interrupt...");

    software_interrupt();

    println!("Software interrupt is handled correctly!");
    print_debug_info();

    println!("The count is {}", count2.borrow());

    println!(
        "The count of count is {} before unregistering.",
        Rc::strong_count(&count2)
    );
    unregister_software_interrupt();
    println!(
        "The count of count is {} after unregistering.",
        Rc::strong_count(&count2)
    );

    loop {
        delay_ms(1000);
    }
}
