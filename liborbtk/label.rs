use super::Widget;

pub struct Label {
    text: String,
}

impl Label {
    pub fn new(text: &str) -> Box<Label> {
        box Label {
            text: text.to_string()
        }
    }
}

impl Widget for Label {}
