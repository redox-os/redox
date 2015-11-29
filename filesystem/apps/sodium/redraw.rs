use std::ops::Range;

#[derive(Clone)]
pub enum RedrawTask {
    Null,
    Lines(Range<usize>),
    LinesAfter(usize),
    Full,
    StatusBar,
    Cursor((usize, usize), (usize, usize)),
}
