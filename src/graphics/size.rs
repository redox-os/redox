#[derive(Copy, Clone)]
pub struct Size {
	pub width: usize,
	pub height: usize
}

impl Size {
	pub fn new(width: usize, height: usize) -> Size {
		Size { width: width, height: height }
	}
}
