use super::Widget;

pub struct ProgressBar {
    pub value: isize,
    pub minimum: isize,
    pub maximum: isize,
}

impl ProgressBar {
    pub fn new(value: isize, minimum: isize, maximum: isize) -> Box<ProgressBar> {
        box ProgressBar {
            value: value,
            minimum: minimum,
            maximum: maximum,
        }
    }
}

impl Widget for ProgressBar {}
