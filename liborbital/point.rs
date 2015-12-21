use std::ops::{Add, Sub};

/// A point
#[derive(Copy, Clone, Debug, Default)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    /// Create a new point
    pub fn new(x: i32, y: i32) -> Self {
        Point { x: x, y: y }
    }
}

impl Add for Point {
    type Output = Point;

    fn add(self, other: Point) -> Self::Output {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Point {
    type Output = Point;

    fn sub(self, other: Point) -> Self::Output {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}
