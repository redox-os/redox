use audio::wav::*;

use graphics::bmp::*;

use programs::common::*;

pub struct Player;

impl Player {
    pub fn new() -> Player {
        Player
    }
}

impl SessionItem for Player {
    fn main(&mut self, url: URL){
        let mut resource = url.open();

        let mut vec: Vec<u8> = Vec::new();
        resource.read_to_end(&mut vec);

        let mut window = Window::new(Point::new((rand() % 400 + 50) as isize, (rand() % 300 + 50) as isize), Size::new(480, 0), "Player (Playing ".to_string() + url.to_string() + ")");
        RedrawEvent { redraw: REDRAW_ALL }.trigger();

        let wav = WAV::from_data(&vec);

        let mut audio = URL::from_string(&"audio://".to_string()).open();
        audio.write(wav.data.as_slice());

        window.title = "Player (".to_string() + url.to_string() + ")";
        RedrawEvent { redraw: REDRAW_ALL }.trigger();

        loop {
            match window.poll() {
                EventOption::Key(key_event) => {
                    if key_event.pressed && key_event.scancode == K_ESC {
                        break;
                    }
                },
                EventOption::None => sys_yield(),
                _ => ()
            }
        }
    }
}
