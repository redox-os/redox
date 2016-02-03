/// A color
#[derive(Copy, Clone)]
#[repr(packed)]
pub struct Color {
    pub data: u32,
}

impl Color {
    /// Create a new color from RGB
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Color { data: 0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32) }
    }

    /// Set the alpha
    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color { data: ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32) }
    }

    // Why are these consts inside an impl?
    pub const BLACK: Color = Color { data: 0xFF000000 };
    pub const BLUE:  Color = Color { data: 0xFF0000FF };
    pub const GREEN: Color = Color { data: 0xFF00FF00 };
    pub const RED:   Color = Color { data: 0xFFFF0000 };
    pub const WHITE: Color = Color { data: 0xFFFFFFFF };
}
