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

        let mut window = Window::new(Point::new((rand() % 400 + 50) as isize,
                                                (rand() % 300 + 50) as isize),
                                     Size::new(320, 0),
                                     "Player (Playing ".to_string() + &url + ")");
        RedrawEvent { redraw: REDRAW_ALL }.trigger();

        let wav = WAV::from_data(&vec);

        let mut audio = File::open(&"audio://".to_string());
        audio.write(wav.data.as_slice());

        window.title = "Player (".to_string() + &url + ")";
        RedrawEvent { redraw: REDRAW_ALL }.trigger();

        loop {
            match window.poll() {
                EventOption::Key(key_event) => {
                    if key_event.pressed && key_event.scancode == K_ESC {
                        break;
                    }
                }
                EventOption::None => sys_yield(),
                _ => (),
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
