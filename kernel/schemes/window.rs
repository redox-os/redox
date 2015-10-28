use alloc::boxed::Box;

use collections::string::String;

use core::{cmp, mem, ptr};

use common::event::Event;
use common::to_num::ToNum;
use common::parse_path::parse_path;

use graphics::display::Display;
use graphics::point::Point;
use graphics::size::Size;
use graphics::window::Window;

use schemes::{KScheme, Resource, ResourceSeek, URL};

/// A window scheme
pub struct WindowScheme;

/// A window resource
pub struct WindowResource {
    /// The window
    pub window: Box<Window>,
    /// Seek point
    pub seek: usize,
}

impl Resource for WindowResource {
    fn dup(&self) -> Option<Box<Resource>> {
        Some(box WindowResource {
            window: Window::new(self.window.point, self.window.size, self.window.title.clone()),
            seek: self.seek,
        })
    }

    /// Return the url of this resource
    fn url(&self) -> URL {
        return URL::from_string(&format!("window://{}/{}/{}/{}/{}",
                                    self.window.point.x,
                                    self.window.point.y,
                                    self.window.size.width,
                                    self.window.size.height,
                                    self.window.title));
    }

    /// Read data to buffer
    fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        //Read events from window
        let mut i = 0;
        while buf.len() - i >= mem::size_of::<Event>() {
            match self.window.poll() {
                Some(event) => {
                    unsafe { ptr::write(buf.as_ptr().offset(i as isize) as *mut Event, event) };
                    i += mem::size_of::<Event>();
                }
                None => break,
            }
        }

        Some(i)
    }

    /// Write to resource
    fn write(&mut self, buf: &[u8]) -> Option<usize> {
        let content = &mut self.window.content;

        let size = cmp::min(content.size - self.seek, buf.len());
        unsafe {
            Display::copy_run(buf.as_ptr() as usize,
                              content.offscreen + self.seek,
                              size);
        }
        self.seek += size;

        return Some(size);
    }

    /// Seek
    fn seek(&mut self, pos: ResourceSeek) -> Option<usize> {
        let end = self.window.content.size;

        self.seek = match pos {
            ResourceSeek::Start(offset) => cmp::min(end, cmp::max(0, offset)),
            ResourceSeek::Current(offset) => cmp::min(end, cmp::max(0, self.seek as isize + offset) as usize),
            ResourceSeek::End(offset) => cmp::min(end, cmp::max(0, end as isize + offset) as usize),
        };

        return Some(self.seek);
    }

    /// Sync the resource, should flip
    fn sync(&mut self) -> bool {
        self.window.redraw();
        true
    }
}

impl KScheme for WindowScheme {
    fn scheme(&self) -> &str {
        "window"
    }

    fn open(&mut self, url: &URL) -> Option<Box<Resource>> {
        //window://host/path/path/path is the path type we're working with.
        let url_path = parse_path(url.reference());
        let pointx = match url_path.get(0) {
            Some(x) => x.to_num_signed(),
            None => 0,
        };
        let pointy = match url_path.get(1) {
            Some(y) => y.to_num_signed(),
            None => 0,
        };
        let size_width = match url_path.get(2) {
            Some(w) => w.to_num(),
            None => 100,
        };
        let size_height = match url_path.get(3) {
            Some(h) => h.to_num(),
            None => 100,
        };

        let mut title = match url_path.get(4) {
            Some(t) => t.clone(),
            None => String::new(),
        };
        for i in 5..url_path.len() {
            if let Some(t) = url_path.get(i) {
                title = title + "/" + t;
            }
        }

        let p: Point = Point::new(pointx, pointy);
        let s: Size = Size::new(size_width, size_height);

        Some(box WindowResource {
            window: Window::new(p, s, title),
            seek: 0,
        })
    }
}
