use super::{Event, Renderer};

use std::any::Any;

pub trait Widget : Any {
    fn draw(&self, renderer: &mut Renderer);
    fn event(&mut self, event: &Event);
}
