#![no_std]
#![no_main]
extern crate alloc;

mod app;

use crate::app::App;
use rust_s3c2440_std::entry;

#[entry]
fn main() -> ! {
    let app = App::new();

    app.main_loop()
}
