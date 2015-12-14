extern crate orbtk;

use orbtk::*;

#[no_mangle] pub fn main() {
    let mut window = Window::new(Rect::new(0, 0, 400, 400), "OrbTK");
    window.widgets.push(Label::new(Rect::new(20, 20, 100, 24), "Test Label"));
    window.exec();
}
