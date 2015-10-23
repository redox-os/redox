/// A color
#[derive(Copy, Clone)]
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

	pub const BLACK: Color = Color { data: 0xFF000000 }; 
	pub const BLUE:  Color = Color { data: 0xFF0000FF }; 
	pub const GREEN: Color = Color { data: 0xFF00FF00 }; 
	pub const RED:   Color = Color { data: 0xFFFF0000 }; 
	pub const WHITE: Color = Color { data: 0xFFFFFFFF };
}


/*
#[derive(Copy, Clone)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub alpha: u8,
}

impl RgbColor {
    /// Create new rgb color
    pub fn new(r: u8, g: u8, b: u8, a: f64) -> RgbColor {
        RgbColor {
            r: r,
            g: g,
            b: b,
            alpha: (a * 255.0).round() as u8,
        }
    }

    /// Set the alpha
    pub fn set_alpha(&mut self, alpha: f64) {
        self.alpha = (alpha * 255.0).round() as u8;
    }

    /// Change alpha
    pub fn alpha(mut self, alpha: f64) -> Self {
        self.alpha = (alpha * 255.0).round() as u8;
        self
    }


    /// Convert the RGB to `Color`
    pub fn to_color(&self) -> Color {
        Color {
            data: ((self.alpha as u32) << 24) |
                  ((self.r as u32) << 16) |
                  ((self.g as u32) << 8) |
                  (self.b as u32),
        }
    }
}
*/
