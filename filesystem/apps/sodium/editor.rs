use super::*;
use redox::*;

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
    /// The prompt
    pub prompt: String,
    /// The settings
    pub options: Options,
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
            cursors: vec![Cursor::new()],
            text: VecDeque::new(),
            scroll_x: 0,
            scroll_y: 0,
            window: *window,
            key_state: KeyState::new(),
            status_bar: StatusBar::new(),
            prompt: String::new(),
            options: Options::new(),
        };

        editor.text.push_back(VecDeque::new());

        // Temporary hacky solution
        editor.text[0].push_back(' ');

        editor.redraw();

        loop {
            let inp = editor.next_inst();
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

    /// Get the leading whitespaces
    pub fn get_indent(&self, n: usize) -> VecDeque<char> {
        let mut ind = VecDeque::new();
        let ln = self.get_ln(n);
        for &c in ln {
            match c {
                '\t' | ' ' => ind.push_back(c),
                _ => break,
            }
        }
        ind
    }
}

