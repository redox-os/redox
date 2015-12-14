use super::{Click, Color, Event, EventOption, MouseEvent, Point, Rect, Renderer, Widget};

use std::sync::Arc;

pub struct Label {
    pub rect: Rect,
    pub text: String,
    pub bg: Color,
    pub fg: Color,
    on_click: Option<Arc<Box<Fn(&mut Label, Point)>>>,
    last_mouse: Option<MouseEvent>,
}

impl Label {
    pub fn new(rect: Rect, text: &str) -> Box<Self> {
        Box::new(Label {
            rect: rect,
            text: text.to_string(),
            bg: Color::rgb(255, 255, 255),
            fg: Color::rgb(0, 0, 0),
            on_click: None,
            last_mouse: None,
        })
    }
}

impl Click for Label {
    fn click(&mut self, point: Point){
        let on_click_option = match self.on_click {
            Some(ref on_click) => Some(on_click.clone()),
            None => None
        };

        if let Some(on_click) = on_click_option {
            on_click(self, point);
        }
    }

    fn on_click(&mut self, func: Box<Fn(&mut Self, Point)>) -> &mut Self {
        self.on_click = Some(Arc::new(func));

        self
    }
}

impl Widget for Label {
    fn draw(&self, renderer: &mut Renderer) {
        renderer.rect(self.rect, self.bg);

        let mut x = 0;
        for c in self.text.chars() {
            if x + 8 <= self.rect.width as isize {
                renderer.char(Point::new(x + self.rect.x, self.rect.y), c, self.fg);
            }
            x += 8;
        }
    }

    fn event(&mut self, event: &Event) {
        match event.to_option() {
            EventOption::Mouse(mouse_event) => {
                let mut click = false;

                if self.rect.contains(Point::new(mouse_event.x, mouse_event.y)){
                    if let Some(last_mouse) = self.last_mouse {
                        if last_mouse.left_button && ! mouse_event.left_button {
                            click = true;
                        }
                    }

                    self.last_mouse = Some(mouse_event);
                } else {
                    self.last_mouse = None;
                }

                if click {
                    let point = Point::new(mouse_event.x - self.rect.x, mouse_event.y - self.rect.y);
                    self.click(point);
                }
            },
            _ => ()
        }
    }
}
