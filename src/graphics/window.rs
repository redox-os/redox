use common::string::*;

use drivers::mouse::*;

use graphics::color::*;
use graphics::display::*;
use graphics::point::*;
use graphics::size::*;

pub struct Window {
    pub point: Point,
    pub size: Size,
    pub title: String,
    pub title_color: Color,
    pub border_color: Color,
    pub content_color: Color,
    pub shaded: bool,
    pub closed: bool,
    pub dragging: bool,
    pub last_mouse_point: Point,
    pub last_mouse_event: MouseEvent
}

impl Window {
    pub fn draw(&self, display: &Display) -> bool {
        if self.closed {
            return false;
        }

        display.rect(Point::new(self.point.x - 2, self.point.y - 18), Size::new(self.size.width + 4, 18), self.border_color);

        let mut cursor = Point::new(self.point.x, self.point.y - 17);
        for character in self.title.as_slice() {
            if cursor.x + 8 <= self.point.x + self.size.width as isize {
                display.char(cursor, *character, self.title_color);
            }
            cursor.x += 8;
        }

        if !self.shaded {
            display.rect(Point::new(self.point.x - 2, self.point.y), Size::new(2, self.size.height), self.border_color);
            display.rect(Point::new(self.point.x - 2, self.point.y + self.size.height as isize), Size::new(self.size.width + 4, 2), self.border_color);
            display.rect(Point::new(self.point.x + self.size.width as isize, self.point.y), Size::new(2, self.size.height), self.border_color);

            display.rect(Point::new(self.point.x, self.point.y), Size::new(self.size.width, self.size.height), self.content_color);
        }

        return true;
    }

    pub fn on_mouse(&mut self, mouse_point: Point, mouse_event: MouseEvent, allow_catch: bool) -> bool{
        let mut caught = false;

        if allow_catch {
            if mouse_event.left_button {
                if ! self.shaded
                    && mouse_point.x >= self.point.x - 2
                    && mouse_point.x < self.point.x + self.size.width as isize + 4
                    && mouse_point.y >= self.point.y - 18
                    && mouse_point.y < self.point.y + self.size.height as isize + 2
                {
                    caught = true;
                }

                if !self.last_mouse_event.left_button
                    && mouse_point.x >= self.point.x - 2
                    && mouse_point.x < self.point.x + self.size.width as isize + 4
                    && mouse_point.y >= self.point.y - 18
                    && mouse_point.y < self.point.y
                {
                    self.dragging = true;
                    caught = true;
                }
            }else{
                self.dragging = false;
            }

            if mouse_event.right_button {
                if ! self.shaded
                    && mouse_point.x >= self.point.x - 2
                    && mouse_point.x < self.point.x + self.size.width as isize + 4
                    && mouse_point.y >= self.point.y - 18
                    && mouse_point.y < self.point.y + self.size.height as isize + 2
                {
                    caught = true;
                }

                if !self.last_mouse_event.right_button
                    && mouse_point.x >= self.point.x - 2
                    && mouse_point.x < self.point.x + self.size.width as isize + 4
                    && mouse_point.y >= self.point.y - 18
                    && mouse_point.y < self.point.y
                {
                    self.shaded = !self.shaded;
                    caught = true;
                }
            }

            if self.dragging {
                self.point.x += mouse_point.x - self.last_mouse_point.x;
                self.point.y += mouse_point.y - self.last_mouse_point.y;
                caught = true;
            }
        }else{
            self.dragging = false;
        }

        self.last_mouse_point = mouse_point;
        self.last_mouse_event = mouse_event;

        return caught;
    }
}