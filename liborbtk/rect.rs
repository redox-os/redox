use super::Point;

#[derive(Clone, Copy, Debug, Default)]
pub struct Rect {
    pub x: isize,
    pub y: isize,
    pub width: usize,
    pub height: usize,
}

impl Rect {
    pub fn new(x: isize, y: isize, width: usize, height: usize) -> Rect {
        Rect {
            x: x,
            y: y,
            width: width,
            height: height,
        }
    }

    pub fn contains(&self, p: Point) -> bool {
        p.x >= self.x && p.x < self.x + self.width as isize &&
        p.y >= self.y && p.y < self.y + self.height as isize
    }
}
