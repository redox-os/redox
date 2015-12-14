use super::{Color, Point, Rect, Renderer, Widget, orbital};

pub struct WindowRenderer<'a> {
    inner: &'a mut Box<orbital::Window>
}

impl<'a> WindowRenderer<'a> {
    pub fn new(inner: &'a mut Box<orbital::Window>) -> WindowRenderer {
        WindowRenderer {
            inner: inner
        }
    }
}

impl<'a> Renderer for WindowRenderer<'a> {
    fn char(&mut self, pos: Point, c: char, color: Color) {
        self.inner.char(pos.x, pos.y, c, color);
    }

    fn rect(&mut self, rect: Rect, color: Color) {
        self.inner.rect(rect.x, rect.y, rect.width, rect.height, color);
    }
}

impl<'a> Drop for WindowRenderer<'a> {
    fn drop(&mut self) {
        self.inner.sync();
    }
}

pub struct Window {
    inner: Box<orbital::Window>,
    pub widgets: Vec<Box<Widget>>,
    pub bg: Color,
}

impl Window {
    pub fn new(rect: Rect, title: &str) -> Box<Self> {
        Box::new(Window {
            inner: orbital::Window::new(rect.x, rect.y, rect.width, rect.height, title).unwrap(),
            widgets: Vec::new(),
            bg: Color::rgb(237, 233, 227),
        })
    }

    pub fn draw(&mut self) {
        self.inner.set(self.bg);

        let mut renderer = WindowRenderer::new(&mut self.inner);
        for widget in self.widgets.iter() {
            widget.draw(&mut renderer);
        }
    }

    pub fn exec(&mut self) {
        self.draw();
        while let Some(event) = self.inner.poll() {
            for mut widget in self.widgets.iter_mut() {
                widget.event(&event);
            }

            self.draw();
        }
    }
}
