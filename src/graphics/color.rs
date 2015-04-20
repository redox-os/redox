#[derive(Copy, Clone)]
pub struct Color {
	pub r: u8,
	pub g: u8,
	pub b: u8
}

impl Color {
	pub fn new(r: u8, g: u8, b: u8) -> Color {
		Color { r: r, g: g, b: b }
	}
}
