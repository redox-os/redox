use core::cmp::max;

use redox::*;

pub fn main() {
    let url = match args().get(1) {
        Some(arg) => arg.clone(),
        None => "none://",
    };


    let mut vec: Vec<u8> = Vec::new();
    if let Some(mut file) = File::open(&url) {
        file.read_to_end(&mut vec);
    }

    let bmp = BMPFile::from_data(&vec);

    let mut window = Window::new((rand() % 400 + 50) as isize,
                                 (rand() % 300 + 50) as isize,
                                 max(320, bmp.width()),
                                 bmp.height(),
                                 &("Viewer (".to_string() + &url + ")")).unwrap();
    window.set(Color::BLACK);
    window.image(0, 0, bmp.width(), bmp.height(), bmp.as_slice());
    window.sync();

    while let Some(event) = window.poll() {
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
