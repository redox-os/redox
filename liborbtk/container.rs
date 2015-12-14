use super::Widget;

pub trait Container : Widget {
    fn children(&self) -> &Vec<Box<Widget>>;
    fn children_mut(&mut self) -> &mut Vec<Box<Widget>>;
}
