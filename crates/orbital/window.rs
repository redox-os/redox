use std::cmp::max;
use std::collections::VecDeque;
use std::mem::size_of;
use std::{ptr, slice};

use super::{Color, Event, Font, Image, Rect};

use system::error::{Error, Result, EINVAL};

pub struct Window {
    pub x: i32,
    pub y: i32,
    image: Image,
    title: String,
    title_image: Image,
    events: VecDeque<Event>,
}

impl Window {
    pub fn new(x: i32, y: i32, w: i32, h: i32, title: String) -> Window {
        let mut title_image = Image::new(title.chars().count() as i32 * 8, 16);
        title_image.as_roi().set(Color::rgba(0, 0, 0, 0));
        {
            let mut x = 0;
            for c in title.chars() {
                title_image.roi(&Rect::new(x, 0, 8, 16)).blend(&Font::render(c, Color::rgb(255, 255, 255)).as_roi());
                x += 8;
            }
        }

        Window {
            x: x,
            y: y,
            image: Image::new(w, h),
            title: title,
            title_image: title_image,
            events: VecDeque::new()
        }
    }

    pub fn width(&self) -> i32 {
        self.image.width()
    }

    pub fn height(&self) -> i32 {
        self.image.height()
    }

    pub fn rect(&self) -> Rect {
        Rect::new(self.x, self.y, self.width(), self.height())
    }

    pub fn title_rect(&self) -> Rect {
        if self.title.is_empty() {
            Rect::default()
        } else {
            Rect::new(self.x, self.y - 18, self.width(), 18)
        }
    }

    pub fn exit_contains(&self, x: i32, y: i32) -> bool {
        ! self.title.is_empty() && x >= max(self.x, self.x + self.width() - 10)  && y >= self.y - 18 && x < self.x + self.width() && y < self.y
    }

    pub fn draw_title(&mut self, image: &mut Image, rect: &Rect, focused: bool) {
        let title_rect = self.title_rect();
        let intersect = rect.intersection(&title_rect);
        if ! intersect.is_empty() {
            if focused {
                image.roi(&intersect).set(Color::rgba(192, 192, 192, 224));
            } else {
                image.roi(&intersect).set(Color::rgba(64, 64, 64, 224));
            }

            {
                let image_rect = Rect::new(title_rect.left() + 4, title_rect.top() + 1, self.title_image.width(), self.title_image.height());
                let image_intersect = intersect.intersection(&image_rect);
                if ! image_intersect.is_empty() {
                    image.roi(&image_intersect).blend(&self.title_image.roi(&image_intersect.offset(-image_rect.left(), -image_rect.top())));
                }
            }

            let x = max(self.x + 2, self.x + self.width() - 10);
            if x + 10 <= self.x + self.width() {
                let mut font_image = Font::render('X', Color::rgb(255, 255, 255));
                let image_rect = Rect::new(title_rect.left() + 4, title_rect.top() + 1, font_image.width(), font_image.height());
                let image_intersect = intersect.intersection(&image_rect);
                if ! image_intersect.is_empty() {
                    image.roi(&image_intersect).blend(&font_image.roi(&image_intersect.offset(-image_rect.left(), -image_rect.top())));
                }
            }
        }
    }

    pub fn draw(&mut self, image: &mut Image, rect: &Rect) {
        let self_rect = self.rect();
        let intersect = rect.intersection(&self_rect);
        if ! intersect.is_empty() {
            image.roi(&intersect).blend(&self.image.roi(&intersect.offset(-self_rect.left(), -self_rect.top())));
        }
    }

    pub fn event(&mut self, event: Event) {
        self.events.push_back(event);
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if buf.len() >= size_of::<Event>() {
            let mut i = 0;
            while i <= buf.len() - size_of::<Event>() {
                if let Some(event) = self.events.pop_front() {
                    unsafe { ptr::write(buf.as_mut_ptr().offset(i as isize) as *mut Event, event) };
                    i += size_of::<Event>();
                } else {
                    break;
                }
            }
            Ok(i)
        } else {
            Err(Error::new(EINVAL))
        }
    }

    pub fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let old = self.image.data_mut();
        let new = unsafe { slice::from_raw_parts(buf.as_ptr() as *const Color, buf.len() / size_of::<Color>()) };

        let mut i = 0;
        while i < old.len() && i < new.len() {
            old[i] = new[i];
            i += 1;
        }

        Ok(i * size_of::<Color>())
    }

    pub fn path(&self, buf: &mut [u8]) -> Result<usize> {
        let mut i = 0;
        let path_str = format!("orbital:/{}/{}/{}/{}/{}", self.x, self.y, self.width(), self.height(), self.title);
        let path = path_str.as_bytes();
        while i < buf.len() && i < path.len() {
            buf[i] = path[i];
            i += 1;
        }
        Ok(i)
    }
}
