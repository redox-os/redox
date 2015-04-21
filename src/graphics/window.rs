use drivers::mouse::*;

use graphics::point::*;
use graphics::size::*;

pub struct Window<'a> {
	pub point: Point,
	pub size: Size,
	pub title: &'a str,
	pub shaded: bool,
	pub dragging: bool,
	pub last_mouse_point: Point,
	pub last_mouse_event: MouseEvent
}


impl<'a> Window<'a> {
	pub fn new(point: Point, size: Size, title: &str) -> Window {
		Window {
            point: point,
            size: size,
            title: title,
            shaded: false,
            dragging: false,
            last_mouse_event: MouseEvent {
                x: 0,
                y: 0,
                left_button: false,
                right_button: false,
                middle_button: false,
                valid: false
            },
            last_mouse_point: Point::new(0, 0)
        }
	}
	
	pub fn on_mouse(&mut self, mouse_point: Point, mouse_event: MouseEvent){
        if mouse_event.left_button
        {
            if !self.last_mouse_event.left_button
                && mouse_point.x >= self.point.x - 2
                && mouse_point.x < self.point.x + self.size.width as i32 + 4
                && mouse_point.y >= self.point.y - 18
                && mouse_point.y < self.point.y
            {
                self.dragging = true;
            }
        }else{
            self.dragging = false;
        }

        if self.dragging {
            self.point.x += mouse_point.x - self.last_mouse_point.x;
            self.point.y += mouse_point.y - self.last_mouse_point.y;
        }

        self.last_mouse_point = mouse_point;
        self.last_mouse_event = mouse_event;
	}
}