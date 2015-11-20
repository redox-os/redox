use redox::*;
use redox::audio::*;

use orbital::*;

pub fn main() {
    let url = match args().get(1) {
        Some(arg) => arg.clone(),
        None => "none://",
    };

    let mut vec: Vec<u8> = Vec::new();
    if let Some(mut file) = File::open(&url) {
        file.read_to_end(&mut vec);
    }

    let mut window = Window::new(-1, -1, 320, 0, &("Player (".to_string() + &url + ")")).unwrap();
    window.sync();

    let wav = WavFile::from_data(&vec);

    if let Some(mut audio) = File::open("audio://") {
        audio.write(&wav.data);
    }

    while let Some(event) = window.poll() {
        if let EventOption::Key(key_event) = event.to_option() {
            if key_event.pressed && key_event.scancode == K_ESC {
                break;
            }
        }
        if let EventOption::Quit(quit_event) = event.to_option() {
            break;
        }
    }
}
