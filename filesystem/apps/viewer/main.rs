extern crate orbclient;

use std::cmp::max;

use std::*;

use orbclient::*;

fn main() {
    let url = match env::args().nth(1) {
        Some(arg) => arg,
        None => "none:".to_string(),
    };

    let bmp = BmpFile::from_path(&url);
    let mut window = Window::new(-1,
                                 -1,
                                 max(320, bmp.width() as u32),
                                 max(32, bmp.height() as u32),
                                 &("Viewer (".to_string() + &url + ")"))
                         .unwrap();
    window.set(Color::rgb(0, 0, 0));
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
