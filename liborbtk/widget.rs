use super::Rect;

use std::any::Any;

pub trait Widget : Any {
    fn rect(&self) -> Rect {
        Rect::default()
    }
}
