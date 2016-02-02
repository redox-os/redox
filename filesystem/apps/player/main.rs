extern crate orbital;

use std::fs::File;
use std::env;
use std::io::{Read, Write};

use wav::WavFile;

use orbital::*;

mod wav;

fn main() {
    let url = match env::args().nth(1) {
        Some(arg) => arg.clone(),
        None => "none:",
    };

    let mut vec: Vec<u8> = Vec::new();
    if let Ok(mut file) = File::open(&url) {
        file.read_to_end(&mut vec);
    }

    let mut window = Window::new(0, 0, 320, 32, &("Player (".to_string() + &url + ")")).unwrap();
    window.sync();

    let wav = WavFile::from_data(&vec);

    if !wav.data.is_empty() {
        if let Ok(mut audio) = File::open("audio://") {
            audio.write(&wav.data);
        }
    }

    loop {
        for event in window.events() {
            if let EventOption::Key(key_event) = event.to_option() {
                if key_event.pressed && key_event.scancode == K_ESC {
                    return;
                }
            }
            if let EventOption::Quit(_) = event.to_option() {
                return;
            }
        }
    }
}
