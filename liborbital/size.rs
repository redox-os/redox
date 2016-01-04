use std::cmp;
use std::ops::{Add, Sub};

/// A size
#[derive(Copy, Clone, Debug, Default)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    /// Create a new size
    pub fn new(width: u32, height: u32) -> Self {
        Size {
            width: width,
            height: height,
        }
    }
}

impl Add for Size {
    type Output = Size;

    fn add(self, other: Size) -> Self::Output {
        Size {
            width: self.width + other.width,
            height: self.height + other.height,
        }
    }
}

impl Sub for Size {
    type Output = Size;

    fn sub(self, other: Size) -> Self::Output {
        Size {
            width: self.width - cmp::min(self.width, other.width),
            height: self.height - cmp::min(self.height, other.height),
        }
    }
}
