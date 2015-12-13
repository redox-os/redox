use super::{Rect};

pub trait Widget {
    fn rect(&self) -> Rect {
        Rect::default()
    }
}
