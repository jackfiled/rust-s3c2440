use alloc::string::ToString;
use rust_s3c2440_hal::nand::{NandFlashController, NandFlashControllerBuilder};
use rust_s3c2440_std::audio::AudioPlayer;
use rust_s3c2440_std::io::get_char;
use rust_s3c2440_std::system::clock::delay_ms;
use rust_s3c2440_std::{print, println};

pub struct App;

impl App {
    pub fn new() -> Self {
        println!("Hello, experiment checker is running!");
        println!("Type the experiment number to run corresponding experiment.");

        Self
    }

    pub fn main_loop(&self) -> ! {
        loop {
            print!(">");

            let input = get_char();
            let char = core::char::from_u32(input as u32);

            if let Some(char) = char {
                println!("{char}");

                match char {
                    '1' => self.experiment1(),
                    '2' => self.experiment2(),
                    '3' => self.experiment3(),
                    _ => println!("Unsupported experiment number: {char}!",),
                }
            } else {
                println!();
                println!("The input {input} is not a valid char.")
            }
        }
    }

    pub fn experiment1(&self) {
        loop {
            let input = get_char();

            if input == b'@' {
                break;
            }

            let char = core::char::from_u32(input as u32);
            if let Some(char) = char {
                if char.is_uppercase() {
                    println!("{}", char.to_lowercase().to_string());
                } else if char.is_lowercase() {
                    println!("{}", char.to_uppercase().to_string());
                } else {
                    println!("\"{}\"", char)
                }
            } else {
                println!("Input {} is not a valid char.", input)
            }
        }
    }

    pub fn experiment2(&self) {
        let controller = NandFlashControllerBuilder::build();

        println!(
            "Nand flash initialized with device ID {:#x}",
            controller.device_id()
        );

        // Try to write and read one block.
        const TARGET_BLOCK: usize = 2025;
        const DATA: &str = "Cross the great wall, come to the world.";
        let address = ((TARGET_BLOCK - 1) * NandFlashController::BLOCK_SIZE).into();

        // println!("Writing block...");
        // controller.write(address, DATA.as_bytes()).unwrap();
        // if controller.write(address, DATA.as_bytes()).is_err() {
        //     println!("Failed to write!");
        //     loop {}
        // }

        println!("Reading block...");
        let mut buffer = [0u8; DATA.len()];

        if controller.read(address, &mut buffer).is_err() {
            println!("Failed to read!");
            loop {}
        }

        match core::str::from_utf8(&buffer) {
            Ok(s) => {
                println!("The reading result is '{s}'.");
            }
            Err(e) => {
                println!("Failed to parse buffer as string: {e}.");
            }
        }
    }

    pub fn experiment3(&self) {
        println!("Music player test, will starting after reading a char.");
        let _ = get_char();
        // let wav_file = include_bytes!("t1_big_endian.wav");
        let wav_file = include_bytes!("t1.wav");
        println!(
            "The base address of wav file: 0x{:x}.",
            wav_file.as_ptr() as usize
        );
        let mut player = AudioPlayer::new();

        let _ = player.play_wav(wav_file);

        loop {
            delay_ms(1000);
        }
    }
}
