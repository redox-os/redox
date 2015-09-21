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

pub fn main(){
    let mut window = Window::new(Point::new((rand() % 400 + 50) as isize, (rand() % 300 + 50) as isize), Size::new(640, 480), "Example Game (Loading)".to_string());
    window.redraw();

    let mut audio = URL::from_str("audio://").open();

    let mut player;
    {
        let mut resource = URL::from_str("file:///game/ninjaroofront.bmp").open();
        let mut bytes: Vec<u8> = Vec::new();
        resource.read_to_end(&mut bytes);
        player = Sprite {
            point: Point::new(200, 200),
            image: unsafe{ BMP::from_data(bytes.as_ptr() as usize) }
        };
    }

    let sound;
    {
        let mut resource = URL::from_str("file:///game/wilhelm.wav").open();
        let mut bytes: Vec<u8> = Vec::new();
        resource.read_to_end(&mut bytes);

        sound = WAV::from_data(&bytes);
    }

    window.title = "Example Game".to_string();
    window.redraw();

    let mut keys: Vec<u8> = Vec::new();
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
                            K_DEL => {
                                window.title = "Example Game (Screaming)".to_string();
                                window.redraw();

                                audio.write(sound.data.as_slice());

                                window.title = "Example Game".to_string();
                                window.redraw();
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
                K_LEFT => player.point.x = max(0, player.point.x - 1),
                K_RIGHT => player.point.x = min(window.content.width as isize - 1, player.point.x + 1),
                K_UP => player.point.y = max(0, player.point.y - 1),
                K_DOWN => player.point.y = min(window.content.height as isize - 1, player.point.y + 1),
                _ => ()
            }
        }

        {
            let content = &mut window.content;
            content.set(Color::new(128, 128, 255));

            player.draw(content);
        }

        window.redraw();

        Duration::new(0, 1000000000/120).sleep();
    }

    window.title = "Example Game (Closing)".to_string();
    RedrawEvent { redraw: REDRAW_ALL }.trigger();

    {
        let mut resource = URL::from_str("file:///game/game_over.wav").open();
        let mut bytes: Vec<u8> = Vec::new();
        resource.read_to_end(&mut bytes);

        let game_over = WAV::from_data(&bytes);
        audio.write(game_over.data.as_slice());
    }
}
