extern crate orbital;

use std::cmp::max;

use std::*;

use orbital::*;

fn main() {
    let url = match env::args().nth(1) {
        Some(arg) => arg.clone(),
        None => "none:",
    };

    let bmp = BmpFile::from_path(url);
    let mut window = Window::new(0,
                                 0,
                                 max(320, bmp.width() as u32),
                                 max(32, bmp.height() as u32),
                                 &("Viewer (".to_string() + &url + ")"))
                         .unwrap();
    window.set(Color::BLACK);
    window.image(0, 0, bmp.width() as u32, bmp.height() as u32, &bmp);
    window.sync();

    loop {
        for event in window.events() {
            if let EventOption::Key(key_event) = event.to_option() {
                if key_event.pressed && key_event.scancode == K_ESC {
                    return;
                }
            }
            if let EventOption::Quit(_) = event.to_option() {
                return;
            }
        }
    }
}
