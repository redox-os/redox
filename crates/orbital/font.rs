use super::Color;
use super::Image;

static FONT: &'static [u8] = include_bytes!("../../filesystem/ui/unifont.font");

pub struct Font;

impl Font {
    pub fn render(character: char, color: Color) -> Image {
        let mut data = Box::new([0; 8*16]);

        let font_i = 16 * (character as usize);
        if font_i + 16 <= FONT.len() {
            for row in 0..16 {
                let row_data = FONT[font_i + row];
                let row_i = row * 8;
                for col in 0..8 {
                    if (row_data >> (7 - col)) & 1 == 1 {
                        data[row_i + col] = color.data;
                    }
                }
            }
        }

        Image::from_data(8, 16, data)
    }
}
