use unbounded_vec::UnboundedVec;

type Grid<T> = UnboundedVec<UnboundedVec<T>>;

#[derive(Clone)]
pub struct Cursor {
    x: usize,
    y: usize,
    hidden: bool,
}

impl Cursor {
    fn x(&self) -> usize {
        self.x + 1
    }

    fn y(&self) -> usize {
        self.y + 1
    }

    fn goto(&mut self, x: usize, y: usize) {
        self.goto_x(x);
        self.goto_y(y);
    }

    fn goto_x(&mut self, x: usize) {
        self.x = x - 1;
    }

    fn goto_y(&mut self, y: usize) {
        self.y = y - 1;
    }
}

pub struct Buffer {
    grid: Grid<char>,
    cursor: Cursor,
    stored_cursor: Cursor,
    scroll: u16,
}

impl Buffer {
    pub fn new() -> Self {
        Buffer {
            grid: Grid::new(),
            cursor: Cursor {
                x: 0,
                y: 0,
                hidden: false,
            },
            stored_cursor: Cursor {
                x: 0,
                y: 0,
                hidden: false,
            },
            scroll: 0,
        }
    }

    pub fn set(&mut self, c: char) {
        self.grid[self.cursor.y][self.cursor.x] = c;
        self.cursor.x += 1;
    }

    pub fn exec(&mut self, action: Action) {
        match action {
            Action::CursorUp(n) => self.cursor.y = self.cursor.y.saturating_sub(n),
            Action::CursorDown(n) => self.cursor.y += n,
            Action::CursorRight(n) => {
                self.cursor.x += n;

                if self.grid[self.cursor.y].len() >= self.cursor.x {
                    self.cursor.y += 1;
                    self.cursor.goto_x(0);
                }
            }
            Action::CursorLeft(n) => self.cursor.x = self.cursor.x.saturating_sub(n),
            Action::NextLine(n) => {
                self.cursor.goto_x(0);
                self.cursor.y += n;
            }
            Action::PreviousLine(n) => {
                self.cursor.y = self.cursor.y.saturating_sub(n);
                self.cursor.x = 0;
            }
            Action::GotoColumn(x) => self.cursor.goto_x(x),
            Action::Goto(x, y) => self.cursor.goto(x, y),
            Action::GotoStart => self.cursor.goto(0, 0),
            Action::EraseAfter => {
                self.grid[self.cursor.y].take(self.cursor.x..);
                self.grid.take(self.cursor.y + 1..);
            }
            Action::EraseBefore => {
                self.grid[self.cursor.y].take(..self.cursor.x - 1);
                self.grid.take(..self.cursor.y);
            }
            Action::EraseAll => {
                self.grid = Grid::new();
            }
            Action::EraseLineAfter => {
                self.grid.take(self.cursor.y..);
            }
            Action::EraseLineBefore => {
                self.grid.take(..self.cursor.y);
            }
            Action::EraseLine => {
                self.grid[self.cursor.y] = UnboundedVec::new();
            }
            Action::ScrollUp(n) => self.cursor.y = self.cursor.y.saturating_sub(1),
            Action::ScrollDown(n) => self.cursor.y += 1,
            Action::Rendition(_) => unimplemented!(), // TODO
            Action::SaveCursor => self.stored_cursor = self.cursor.clone(),
            Action::RestoreCursor => self.cursor = self.stored_cursor.clone(),
            Action::HideCursor => self.cursor.hide = true,
            Action::ShowCursor => self.cursor.hide = false,
            Action::NewLine => {
                self.cursor.goto_x(0);
                self.cursor.y += 1;
            }
            Action::CarriageReturn => self.cursor.goto_x(0),
            _ => unimplemented!(),

        }
    }
}
