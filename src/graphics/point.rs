use core::ops::Add;
use core::ops::Sub;

#[derive(Copy, Clone)]
pub struct Point {
    pub x: isize,
    pub y: isize
}

impl Point {
    pub fn new(x: isize, y: isize) -> Point {
        Point { x: x, y: y }
    }
}

impl Add for Point {
    type Output = Point;

    fn add(self, other: Point) -> Self::Output {
        Point {
            x: self.x + other.x,
            y: self.y + other.y
        }
    }
}

impl Sub for Point {
    type Output = Point;

    fn sub(self, other: Point) -> Self::Output {
        Point {
            x: self.x - other.x,
            y: self.y - other.y
        }
    }
}
