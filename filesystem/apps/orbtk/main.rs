extern crate orbtk;

use orbtk::*;

#[no_mangle] pub fn main() {
    let mut window = Window::new(Rect::new(0, 0, 400, 400), "OrbTK");

    let mut label = Label::new(Rect::new(20, 20, 80, 16), "Test Label");
    label.on_click(Box::new(|label: &mut Label, point: Point| {
        label.text = format!("{:?}", point);
        label.rect.width = label.text.chars().count() * 8;
    }));
    window.widgets.push(label);

    let mut progress_bar = ProgressBar::new(Rect::new(20, 60, 200, 16), 50);
    progress_bar.on_click(Box::new(|progress_bar: &mut ProgressBar, point: Point| {
        progress_bar.value = point.x * 100 / progress_bar.rect.width as isize;
    }));
    window.widgets.push(progress_bar);

    window.exec();
}
