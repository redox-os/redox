pub use self::graphic::GraphicScreen;
pub use self::text::TextScreen;

mod graphic;
mod text;

pub enum Screen {
    Graphic(GraphicScreen),
    Text(TextScreen)
}
