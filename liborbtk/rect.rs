use super::{Point, Size};

#[derive(Clone, Copy, Debug, Default)]
pub struct Rect {
    pub pos: Point,
    pub size: Size,
}

impl Rect {
    pub fn new(pos: Point, size: Size) -> Rect {
        Rect {
            pos: pos,
            size: size,
        }
    }

    pub fn contains(&self, p: Point) -> bool {
        p.x >= self.pos.x && p.x < self.pos.x + self.size.width &&
        p.y >= self.pos.y && p.y < self.pos.y + self.size.height
    }
}
