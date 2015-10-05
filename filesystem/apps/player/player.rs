use redox::*;

pub struct Player;

impl Player {
    pub fn new() -> Player {
        Player
    }

    fn main(&mut self, url: String) {
        let mut resource = File::open(&url);

        let mut vec: Vec<u8> = Vec::new();
        resource.read_to_end(&mut vec);

        let mut window = File::open(&("window://".to_string()
                                    + "/" + (rand() % 400 + 50)
                                    + "/" + (rand() % 300 + 50)
                                    + "/320"
                                    + "/0"
                                    + "/Player (Playing " + &url + ")"));
        window.sync();

        let wav = WAV::from_data(&vec);

        let mut audio = File::open(&"audio://".to_string());
        audio.write(wav.data.as_slice());

        loop {
            let mut event_slice = Event::slice();
            match window.read(&mut event_slice) {
                Option::Some(_) => {
                    match Event::from_slice(&event_slice).to_option() {
                        EventOption::Key(key_event) => {
                            if key_event.pressed && key_event.scancode == K_ESC {
                                break;
                            }
                        }
                        _ => (),
                    }
                },
                Option::None => break
            }
        }
    }
}

pub fn main() {
    match args().get(1) {
        Option::Some(arg) => Player::new().main(arg.clone()),
        Option::None => Player::new().main("none://".to_string()),
    }
}
