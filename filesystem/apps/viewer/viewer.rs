use core::cmp::max;

use redox::*;

pub fn main() {
    let url = match args().get(1) {
        Option::Some(arg) => arg.clone(),
        Option::None => "none://".to_string(),
    };

    let mut resource = File::open(&url);

    let mut vec: Vec<u8> = Vec::new();
    resource.read_to_end(&mut vec);

    let image = BMP::from_data(&vec);

    let mut window = File::open(&("window://".to_string()
                                + "/" + (rand() % 400 + 50)
                                + "/" + (rand() % 300 + 50)
                                + "/" + max(320, image.size.width)
                                + "/" + image.size.height
                                + "/Viewer (" + &url + ")"));
    window.write(image.as_slice());
    window.sync();

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
