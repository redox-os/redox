extern crate orbital;

use orbital::Color;

use window::ConsoleWindow;

mod window;

#[no_mangle] pub fn main() {
    let mut window = ConsoleWindow::new(-1, -1, 576, 400, "Terminal");

    loop {
        window.print("# ", Color::rgb(255, 255, 255));
        if let Some(line) = window.read() {
            window.print(&format!("{}\n", line), Color::rgb(224, 224, 224));
        } else {
            break;
        }
    }
}
