/// A color
#[derive(Copy, Clone)]
#[repr(packed)]
pub struct Color {
    pub data: u32,
}

impl Color {
    /// Create a new color from RGB
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Color { data: 0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32) }
    }

    pub fn ansi(value: u8) -> Self {
        match value {
            0 => Color::new(0x00, 0x00, 0x00),
            1 => Color::new(0x80, 0x00, 0x00),
            2 => Color::new(0x00, 0x80, 0x00),
            3 => Color::new(0x80, 0x80, 0x00),
            4 => Color::new(0x00, 0x00, 0x80),
            5 => Color::new(0x80, 0x00, 0x80),
            6 => Color::new(0x00, 0x80, 0x80),
            7 => Color::new(0xc0, 0xc0, 0xc0),
            8 => Color::new(0x80, 0x80, 0x80),
            9 => Color::new(0xff, 0x00, 0x00),
            10 => Color::new(0x00, 0xff, 0x00),
            11 => Color::new(0xff, 0xff, 0x00),
            12 => Color::new(0x00, 0x00, 0xff),
            13 => Color::new(0xff, 0x00, 0xff),
            14 => Color::new(0x00, 0xff, 0xff),
            15 => Color::new(0xff, 0xff, 0xff),
            16 ... 231 => {
                let convert = |value: u8| -> u8 {
                    match value {
                        0 => 0,
                        _ => value * 0x28 + 0x28
                    }
                };

                let r = convert((value - 16)/36 % 6);
                let g = convert((value - 16)/6 % 6);
                let b = convert((value - 16) % 6);
                Color::new(r, g, b)
            },
            232 ... 255 => {
                let gray = (value - 232) * 10 + 8;
                Color::new(gray, gray, gray)
            },
            _ => Color::new(0, 0, 0)
        }
    }
}
