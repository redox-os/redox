use super::{Click, Color, Event, EventOption, MouseEvent, Point, Rect, Renderer, Widget};

use std::cmp::{min, max};
use std::sync::Arc;

pub struct ProgressBar {
    pub rect: Rect,
    pub value: isize,
    pub minimum: isize,
    pub maximum: isize,
    pub bg: Color,
    pub fg: Color,
    on_click: Option<Arc<Box<Fn(&mut ProgressBar, Point)>>>,
    last_mouse: Option<MouseEvent>,
}

impl ProgressBar {
    pub fn new(rect: Rect, value: isize) -> Box<Self> {
        Box::new(ProgressBar {
            rect: rect,
            value: value,
            minimum: 0,
            maximum: 100,
            bg: Color::rgb(255, 255, 255),
            fg: Color::rgb(65, 139, 212),
            on_click: None,
            last_mouse: None,
        })
    }
}

impl Click for ProgressBar {
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

impl Widget for ProgressBar {
    fn draw(&self, renderer: &mut Renderer) {
        renderer.rect(self.rect, self.bg);
        renderer.rect(Rect::new(
            self.rect.x,
            self.rect.y,
            ((self.rect.width as isize * max(0, min(self.maximum, self.value - self.minimum)))/max(1, self.maximum - self.minimum)) as usize,
            self.rect.height
        ), self.fg);
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
