use super::*;
use redox::*;

#[derive(Copy, Clone)]
/// An instruction
pub struct Inst(pub u16, pub Key);

/// The state of the editor
pub struct Editor {
    /// The current cursor
    pub current_cursor: u8,
    /// The cursors
    pub cursors: Vec<Cursor>,
    /// The text (document)
    pub text: VecDeque<VecDeque<char>>,
    /// The x coordinate of the scroll
    pub scroll_x: usize,
    /// The y coordinate of the scroll
    pub scroll_y: usize,
    /// The window
    pub window: Window,
    /// The key state
    pub key_state: KeyState,
    /// The status bar
    pub status_bar: StatusBar,
}

impl Editor {
    /// Create new default state editor
    pub fn new() -> Editor {

        let window = Window::new((rand() % 400 + 50) as isize,
                                 (rand() % 300 + 50) as isize,
                                 700,
                                 500,
                                 &"Sodium").unwrap();

        let mut editor = Editor {
            current_cursor: 0,
            cursors: Vec::new(),
            text: VecDeque::new(),
            scroll_x: 0,
            scroll_y: 0,
            window: *window,
            key_state: KeyState::new(),
        };

        editor.cursors.push(Cursor::new());
        editor.text.push_back(VecDeque::new());

        loop {
            let inp = next_inst(&mut editor);
            editor.exec(inp);
            editor.redraw();
        }

        editor
    }
}

