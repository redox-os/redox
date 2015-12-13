#[derive(Clone, Copy, Debug, Default)]
pub struct Size {
    pub width: isize,
    pub height: isize,
}

impl Size {
    pub fn new(width: isize, height: isize) -> Size {
        Size {
            width: width,
            height: height,
        }
    }
}
