#![deny(warnings)]

use std::fs::File;
use std::env;
use std::io::{Read, Write};

use wav::WavFile;

mod wav;

fn main() {
    match env::args().nth(1) {
        Some(arg) => match File::open(&arg) {
            Ok(mut file) => {
                let mut vec: Vec<u8> = Vec::new();
                file.read_to_end(&mut vec).unwrap();

                let wav = WavFile::from_data(&vec);

                println!("WAV: {} Channels {} Hz {} Depth {} Bytes", wav.channels, wav.sample_rate, wav.sample_bits, wav.data.len());

                match File::open("audio:") {
                    Ok(mut audio) => match audio.write(&wav.data) {
                        Ok(_) => (),
                        Err(err) => println!("play: failed to write to audio: {}", err)
                    },
                    Err(err) => println!("play: failed to open audio: {}", err)
                }
            },
            Err(err) => println!("play: failed to open {}: {}", arg, err)
        },
        None => println!("play [WAV file]")
    }
}
