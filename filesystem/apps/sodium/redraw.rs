use std::ops::Range;

#[derive(Clone)]
/// A task for the renderer for redrawing
pub enum RedrawTask {
    Null,
    Lines(Range<usize>),
    LinesAfter(usize),
    Full,
    StatusBar,
    Cursor((usize, usize), (usize, usize)),
}
