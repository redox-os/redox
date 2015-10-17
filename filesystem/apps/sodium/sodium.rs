// TODO:
//      - Simplify using instruction iterators
//      - Make movement mode
//      - Record modifiers

mod editor;
pub use self::editor::*;

mod parse;
pub use self::parse::*;

mod mode;
pub use self::mode::*;

mod movement;
pub use self::movement::*;

mod cursor;
pub use self::cursor::*;

mod insert;
pub use self::insert::*;

mod exec;
pub use self::exec::*;

use redox::*;

pub fn main() {
    let mut window = Window::new((rand() % 400 + 50) as isize,
                                 (rand() % 300 + 50) as isize,
                                 576,
                                 400,
                                 &"Sodium").unwrap();

    let mut editor = Editor::new();

    let mut inp = window.event_iter().map(|x| {
        x.to_option()
    }).inst_iter(&mut editor);

    inp.exec();
    window.set([255, 255, 255, 255]);


}

pub fn redraw() {
    /*
                    // Redraw window
                    window.set([255, 255, 255, 255]);

                    for (y, row) in editor.text.iter().enumerate() {
                        for (x, c) in row.iter().enumerate() {
                            window.char(8 * (y - editor.scroll_y) as isize, 16 * (x - editor.scroll_x) as isize, *c, [128, 128, 128, 255]);
                            if editor.cursor().x == x && editor.cursor().y == y {
                                window.char(8 * (y - editor.scroll_y) as isize, 16 * (x - editor.scroll_x) as isize, '_', [128, 128, 128, 255]);
                            }
                        }
                    }
    */
}
