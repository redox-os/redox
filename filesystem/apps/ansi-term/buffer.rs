use unbounded_vec::UnboundedVec;

type Grid<T> = UnboundedVec<UnboundedVec<T>>;

pub struct Cursor {
    x: usize,
    y: usize,
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
}
