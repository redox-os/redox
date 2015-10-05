use core::cmp::min;
use core::ops::Add;
use core::ops::Sub;

/// A size
#[derive(Copy, Clone)]
pub struct Size {
    pub width: usize,
    pub height: usize,
}

impl Size {
    /// Create a new size
    pub fn new(width: usize, height: usize) -> Size {
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
            width: self.width - min(self.width, other.width),
            height: self.height - min(self.height, other.height),
        }
    }
}
