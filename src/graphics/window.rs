use graphics::point::*;
use graphics::size::*;

pub struct Window<'a> {
	pub point: Point,
	pub size: Size,
	pub title: &'a str
}


impl<'a> Window<'a> {
	pub fn new(point: Point, size: Size, title: &str) -> Window {
		Window { point:point, size:size, title:title }
	}
}