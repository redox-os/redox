use redox::*;

pub fn main() {
    let url = match args().get(1) {
        Option::Some(arg) => arg.clone(),
        Option::None => "none://",
    };

    let mut vec: Vec<u8> = Vec::new();
    if let Some(mut file) = File::open(&url) {
        file.read_to_end(&mut vec);
    }

    let mut window = Window::new((rand() % 400 + 50) as isize,
                                 (rand() % 300 + 50) as isize,
                                 320,
                                 0,
                                 &("Player (".to_string() + &url + ")")).unwrap();
    window.sync();

    let wav = WAV::from_data(&vec);

    if let Some(mut audio) = File::open("audio://") {
        audio.write(&wav.data);
    }

    while let Option::Some(event) = window.poll() {
        match event.to_option() {
            EventOption::Key(key_event) => {
                if key_event.pressed && key_event.scancode == K_ESC {
                    break;
                }
            }
            _ => (),
        }
    }
}
