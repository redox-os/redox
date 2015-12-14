extern crate orbtk;

use orbtk::*;

#[no_mangle] pub fn main() {
    let mut window = Window::new(Point::default(), Size::new(400, 400), "OrbTK");
    window.exec();
}
