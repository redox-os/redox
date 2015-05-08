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
