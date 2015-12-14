use super::{Color, Event, EventOption, MouseEvent, Point, Rect, Renderer, Widget};

pub struct Label {
    pub rect: Rect,
    pub text: String,
    pub bg: Color,
    pub fg: Color,
    last_mouse: Option<MouseEvent>,
}

impl Label {
    pub fn new(rect: Rect, text: &str) -> Box<Label> {
        box Label {
            rect: rect,
            text: text.to_string(),
            bg: Color::rgb(255, 255, 255),
            fg: Color::rgb(0, 0, 0),
            last_mouse: None,
        }
    }
}

impl Widget for Label {
    fn draw(&self, renderer: &mut Renderer) {
        renderer.rect(self.rect, self.bg);

        let mut x = 0;
        for c in self.text.chars() {
            if x + 8 < self.rect.width as isize {
                renderer.char(Point::new(x + self.rect.x, self.rect.y), c, self.fg);
            }
            x += 8;
        }
    }

    fn event(&mut self, event: &Event) {
        match event.to_option() {
            EventOption::Mouse(mouse_event) => {
                if self.rect.contains(Point::new(mouse_event.x, mouse_event.y)){
                    if let Some(last_mouse) = self.last_mouse {
                        if last_mouse.left_button && ! mouse_event.left_button {
                            self.bg = Color::rgb(192, 192, 192);
                        }
                    }

                    self.last_mouse = Some(mouse_event);
                }
            },
            _ => ()
        }
    }
}
