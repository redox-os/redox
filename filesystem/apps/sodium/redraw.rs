use core::ops::Range;

pub enum RedrawTask {
    Null,
    Lines(Range<usize>),
    Full,
    StatusBar,
    Cursor((usize, usize), (usize, usize)),
}

