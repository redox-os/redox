use redox::*;

pub fn main() {
    let url = match args().get(1) {
        Option::Some(arg) => arg.clone(),
        Option::None => "none://".to_string(),
    };

    let mut resource = File::open(&url);

    let mut vec: Vec<u8> = Vec::new();
    resource.read_to_end(&mut vec);



    let mut window = Window::new((rand() % 400 + 50) as isize, (rand() % 300 + 50) as isize,
                                320, 0,
                                &("Player (".to_string() + &url + ")"));
    window.sync();

    let wav = WAV::from_data(&vec);

    let mut audio = File::open(&"audio://".to_string());
    audio.write(wav.data.as_slice());

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
