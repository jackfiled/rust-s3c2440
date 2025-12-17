#![no_std]
#![no_main]
extern crate alloc;

use rust_s3c2440_std::audio::AudioPlayer;
use rust_s3c2440_std::io::get_char;
use rust_s3c2440_std::system::clock::delay_ms;
use rust_s3c2440_std::{entry, println};

#[entry]
fn entry() -> ! {
    main()
}

fn main() -> ! {
    println!("Music player test, will starting after reading a char.");
    let _ = get_char();
    let wav_file = include_bytes!("t1_big_endian.wav");
    let mut player = AudioPlayer::new();

    let _ = player.play_wav(wav_file);

    loop {
        delay_ms(1000);
    }
}
