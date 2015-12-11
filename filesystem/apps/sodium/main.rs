extern crate orbital;

pub use std::collections::VecDeque;

mod editor;
pub use self::editor::*;

mod parse;
pub use self::parse::*;

mod key_state;
pub use self::key_state::*;

mod key;
pub use self::key::*;

mod prompt;
pub use self::prompt::*;

mod open;
pub use self::open::*;

mod redraw;
pub use self::redraw::*;

mod options;
pub use self::options::*;

mod position;
pub use self::position::*;

mod graphics;
pub use self::graphics::*;

mod selection;
pub use self::selection::*;

mod mode;
pub use self::mode::*;

mod movement;
pub use self::movement::*;

mod motion;
pub use self::motion::*;

mod cursor;
pub use self::cursor::*;

mod insert;
pub use self::insert::*;

mod delete;
pub use self::delete::*;

mod exec;
pub use self::exec::*;

pub mod invert;

#[no_mangle] pub fn main() {
    Editor::init();
}
