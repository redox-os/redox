#[cfg(target_os = "redox")]
extern crate orbital;

#[cfg(target_os = "redox")]
use self::orbital::Window as InnerWindow;

use super::{Container, Point, Size, Widget};

pub struct Window {
    inner: Box<InnerWindow>,
    widgets: Vec<Box<Widget>>
}

impl Window {
    pub fn new(pos: Point, size: Size, title: &str) -> Box<Window> {
        box Window {
            inner: InnerWindow::new(pos.x, pos.y, size.width as usize, size.height as usize, title).unwrap(),
            widgets: Vec::new()
        }
    }

    pub fn exec(&mut self) {
        while let Some(event) = self.inner.poll() {
            println!("{:?}", event.to_option());
        }
    }
}

impl Widget for Window {}

impl Container for Window {
    fn children(&self) -> &Vec<Box<Widget>> {
        &self.widgets
    }

    fn children_mut(&mut self) -> &mut Vec<Box<Widget>> {
        &mut self.widgets
    }
}
