#![no_std]
#![no_main]
extern crate alloc;

use rust_s3c2440_std::audio::AudioPlayer;
use rust_s3c2440_std::io::get_char;
use rust_s3c2440_std::system::clock::delay_ms;
use rust_s3c2440_std::{MANAGER, entry, println};

#[entry]
fn main() -> ! {
    println!("Enable interrupt.");
    MANAGER
        .get()
        .unwrap()
        .interrupt()
        .borrow_mut()
        .enable_interrupt();

    println!("Music player test, will starting after reading a char.");
    let _ = get_char();
    let wav_file = include_bytes!("tone_440hz_22050_stereo.wav");
    // let wav_file = include_bytes!("beep_silence_22050_stereo.wav");
    let mut player = AudioPlayer::new();

    let _ = player.play_wav(wav_file);

    loop {
        delay_ms(1000);
    }
}
