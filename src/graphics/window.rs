use core::str::StrExt;

use drivers::mouse::*;

use graphics::color::*;
use graphics::display::*;
use graphics::point::*;
use graphics::size::*;

pub struct Window {
	pub point: Point,
	pub size: Size,
	pub title: &'static str,
	pub shaded: bool,
	pub dragging: bool,
	pub last_mouse_point: Point,
	pub last_mouse_event: MouseEvent
}


impl Window {	
	pub fn draw(&self, display: &Display) {
        let border_color = Color::new(0, 0, 0);
        let title_color = Color::new(255, 255, 255);
		display.rect(Point::new(self.point.x - 2, self.point.y - 18), Size::new(self.size.width + 4, 18), border_color);
		
        let mut cursor = Point::new(self.point.x, self.point.y - 17);
        for character in self.title.chars() {
            if cursor.x + 8 <= self.point.x + self.size.width as i32 {
                display.char(cursor, character, title_color);
            }
            cursor.x += 8;
        }
		
		if !self.shaded {
            display.rect(Point::new(self.point.x - 2, self.point.y), Size::new(2, self.size.height), border_color);
            display.rect(Point::new(self.point.x - 2, self.point.y + self.size.height as i32), Size::new(self.size.width + 4, 2), border_color);
            display.rect(Point::new(self.point.x + self.size.width as i32, self.point.y), Size::new(2, self.size.height), border_color);
            
            display.rect(Point::new(self.point.x, self.point.y), Size::new(self.size.width, self.size.height), Color { r: 0, g: 0, b: 0, a:196 });
		}
	}
	
	pub fn on_mouse(&mut self, mouse_point: Point, mouse_event: MouseEvent) -> bool{
        let mut caught = false;
	
        if mouse_event.left_button
        {
            if !self.last_mouse_event.left_button
                && mouse_point.x >= self.point.x - 2
                && mouse_point.x < self.point.x + self.size.width as i32 + 4
                && mouse_point.y >= self.point.y - 18
                && mouse_point.y < self.point.y
            {
                self.dragging = true;
                caught = true;
            }
        }else{
            self.dragging = false;
        }
        
        if mouse_event.right_button
        {
            if !self.last_mouse_event.right_button
                && mouse_point.x >= self.point.x - 2
                && mouse_point.x < self.point.x + self.size.width as i32 + 4
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

        self.last_mouse_point = mouse_point;
        self.last_mouse_event = mouse_event;
        
        return caught;
	}
}