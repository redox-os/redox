use super::*;

use redox::*;

use orbital::*;

/// The current state of the editor, including the file, the cursor, the scrolling info, etc.
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
    /// The status bar
    pub status_bar: StatusBar,
    /// The prompt
    pub prompt: String,
    /// The settings
    pub options: Options,
    /// The key state
    pub key_state: KeyState,
    /// Redraw
    pub redraw_task: RedrawTask,
}

impl Editor {
    /// Create new default state editor
    pub fn new() -> Editor {


        let window = Window::new(-1, -1, 700, 500, &"Sodium").unwrap();

        let mut editor = Editor {
            current_cursor: 0,
            cursors: vec![Cursor::new()],
            text: VecDeque::new(),
            scroll_x: 0,
            scroll_y: 0,
            window: *window,
            status_bar: StatusBar::new(),
            prompt: String::new(),
            options: Options::new(),
            key_state: KeyState::new(),
            redraw_task: RedrawTask::Null,
        };

        editor.text.push_back(VecDeque::new());


        editor.redraw();

        loop {
            let inp = editor.get_inst();
            editor.exec(inp);
            editor.redraw();
            editor.status_bar.mode = editor.cursor().mode.to_string();
        }

        editor
    }

    /// Get a slice of the current line
    pub fn get_ln(&self, n: usize) -> &[char] {
        self.text[n].as_slices().0
    }

    /// Get the leading whitespaces of the current line. Used for autoindenting.
    pub fn get_indent(&self, n: usize) -> &[char] {
        let ln = self.get_ln(n);
        let mut len = 0;
        for &c in ln {
            match c {
                '\t' | ' ' => len += 1,
                _ => break,
            }
        }
        &ln[..len]
    }
}
