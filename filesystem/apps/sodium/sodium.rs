// TODO:
//      - Simplify using instruction iterators
//      - Make movement mode
//      - Record modifiers

mod editor;
pub use self::editor::*;

mod parse;
pub use self::parse::*;

mod graphics;
pub use self::graphics::*;

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


pub fn main() {

    let mut editor = Editor::new();


}

