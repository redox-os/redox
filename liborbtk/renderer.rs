use super::{Color, Point, Rect};

pub trait Renderer {
    fn char(&mut self, pos: Point, c: char, color: Color);
    fn rect(&mut self, rect: Rect, color: Color);
}
