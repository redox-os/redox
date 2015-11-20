use core::cmp::max;

use redox::*;

use orbital::*;

pub fn main() {
    let url = match args().get(1) {
        Some(arg) => arg.clone(),
        None => "none://",
    };

    let bmp = BmpFile::from_path(url);
    let mut window = Window::new(-1,
                                 -1,
                                 max(320, bmp.width()),
                                 bmp.height(),
                                 &("Viewer (".to_string() + &url + ")"))
                         .unwrap();
    window.set(Color::BLACK);
    window.image(0, 0, bmp.width(), bmp.height(), &bmp);
    window.sync();

    while let Some(event) = window.poll() {
        if let EventOption::Key(key_event) = event.to_option() {
            if key_event.pressed && key_event.scancode == K_ESC {
                break;
            }
        }
        if let EventOption::Quit(quit_event) = event.to_option() {
            break;
        }
    }
}
