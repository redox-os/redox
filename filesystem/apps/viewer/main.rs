extern crate orbital;

use std::cmp::max;

use std::*;

use orbital::*;

#[no_mangle]
pub fn main() {
    let url = match env::args().nth(1) {
        Some(arg) => arg.clone(),
        None => "none:",
    };

    let bmp = BmpFile::from_path(url);
    let mut window = Window::new(-1,
                                 -1,
                                 max(320, bmp.width() as u32),
                                 bmp.height() as u32,
                                 &("Viewer (".to_string() + &url + ")"))
                         .unwrap();
    window.set(Color::BLACK);
    window.image(0, 0, bmp.width() as u32, bmp.height() as u32, &bmp);
    window.sync();

    while let Some(event) = window.poll() {
        if let EventOption::Key(key_event) = event.to_option() {
            if key_event.pressed && key_event.scancode == K_ESC {
                break;
            }
        }
        if let EventOption::Quit(_) = event.to_option() {
            break;
        }
    }
}
