use audio::wav::*;

use programs::common::*;

use graphics::bmp::*;

pub struct Sprite {
    point: Point,
    image: BMP
}

impl Sprite {
    pub fn draw(&self, content: &mut Display){
        content.image_alpha(self.point, self.image.data, self.image.size);
    }
}

pub struct Application;

impl SessionItem for Application {
    fn main(&mut self, url: URL){
        let mut window = Window::new(Point::new((rand() % 400 + 50) as isize, (rand() % 300 + 50) as isize), Size::new(640, 480), "Example Game (Loading)".to_string());

        let mut player;
        {
            let mut resource = URL::from_string(&"file:///game/ninjaroofront.bmp".to_string()).open();
            let mut bytes: Vec<u8> = Vec::new();
            resource.read_to_end(&mut bytes);
            player = Sprite {
                point: Point::new(200, 200),
                image: unsafe{ BMP::from_data(bytes.as_ptr() as usize) }
            };
        }

        window.title = "Example Game".to_string();

        let sound;
        {
            let mut resource = URL::from_string(&"file:///game/wilhelm.wav".to_string()).open();
            let mut bytes: Vec<u8> = Vec::new();
            resource.read_to_end(&mut bytes);

            sound = WAV::from_data(&bytes);
        }

        let mut keys: Vec<u8> = Vec::new();
        let mut redraw = true;
        let mut running = true;
        while running {
            loop {
                match window.poll() {
                    EventOption::Key(key_event) => {
                        if key_event.pressed {
                            match key_event.scancode {
                                K_ESC => {
                                    running = false;
                                    break;
                                },
                                K_CTRL => {
                                    let mut resource = URL::from_string(&"audio://".to_string()).open();
                                    resource.write(sound.data.as_slice());
                                },
                                _ => ()
                            }

                            let mut found = false;
                            for key in keys.iter() {
                                if *key == key_event.scancode {
                                    found = true;
                                    break;
                                }
                            }
                            if ! found {
                                keys.push(key_event.scancode);
                            }
                        }else{
                            let mut i = 0;
                            while i < keys.len() {
                                let mut remove = false;
                                if let Option::Some(key) = keys.get(i) {
                                    if *key == key_event.scancode {
                                        remove = true;
                                    }
                                }
                                if remove {
                                    keys.remove(i);
                                }else{
                                    i += 1;
                                }
                            }
                        }
                    },
                    EventOption::None => break,
                    _ => ()
                }
            }

            for key in keys.iter() {
                match *key {
                    K_LEFT => {
                        player.point.x = max(0, player.point.x - 1);
                        redraw = true;
                    },
                    K_RIGHT => {
                        player.point.x = min(window.content.width as isize - 1, player.point.x + 1);
                        redraw = true;
                    },
                    K_UP => {
                        player.point.y = max(0, player.point.y - 1);
                        redraw = true;
                    },
                    K_DOWN => {
                        player.point.y = min(window.content.height as isize - 1, player.point.y + 1);
                        redraw = true;
                    },
                    _ => ()
                }
            }

            if redraw {
                redraw = false;

                let content = &mut window.content;
                content.set(Color::new(128, 128, 255));

                player.draw(content);

                content.flip();

                RedrawEvent {
                    redraw: REDRAW_ALL
                }.trigger();
            }

            Duration::new(0, 1000000000/120).sleep();
        }
    }
}

impl Application {
    pub fn new() -> Application {
        Application
    }
}
