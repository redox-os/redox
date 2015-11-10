use redox::{Box, String, Url};
use redox::{cmp, mem, ptr};
use redox::io::*;
use redox::ops::DerefMut;
use redox::to_num::ToNum;

use orbital::event::Event;
use orbital::Point;
use orbital::Size;

use self::display::Display;
use self::session::Session;
use self::window::Window;

pub mod display;
pub mod package;
pub mod scheduler;
pub mod session;
pub mod window;

pub static mut session_ptr: *mut Session = 0 as *mut Session;

/// A window scheme
pub struct Scheme {
    pub session: Box<Session>,
}

/// A window resource
pub struct Resource {
    /// The window
    pub window: Box<Window>,
    /// Seek point
    pub seek: usize,
}

impl Resource {
    pub fn dup(&self) -> Option<Box<Resource>> {
        Some(box Resource {
            window: Window::new(self.window.point, self.window.size, self.window.title.clone()),
            seek: self.seek,
        })
    }

    /// Return the url of this resource
    pub fn path(&self) -> Option<String> {
        Some(format!("window://{}/{}/{}/{}/{}",
                         self.window.point.x,
                         self.window.point.y,
                         self.window.size.width,
                         self.window.size.height,
                         self.window.title))
    }

    /// Read data to buffer
    pub fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
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
    pub fn write(&mut self, buf: &[u8]) -> Option<usize> {
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
    pub fn seek(&mut self, pos: SeekFrom) -> Option<usize> {
        let end = self.window.content.size;

        self.seek = match pos {
            SeekFrom::Start(offset) => cmp::min(end, cmp::max(0, offset)),
            SeekFrom::Current(offset) => cmp::min(end, cmp::max(0, self.seek as isize + offset) as usize),
            SeekFrom::End(offset) => cmp::min(end, cmp::max(0, end as isize + offset) as usize),
        };

        return Some(self.seek);
    }

    /// Sync the resource, should flip
    pub fn sync(&mut self) -> bool {
        self.window.redraw();
        true
    }
}

impl Scheme {
    pub fn new() -> Box<Scheme> {
        let mut ret = box Scheme {
            session: Session::new()
        };
        unsafe { session_ptr = ret.session.deref_mut() };
        ret
    }

    pub fn open(&mut self, url_str: &str, _: usize) -> Option<Box<Resource>> {
        //window://host/path/path/path is the path type we're working with.
        let url_path = Url::from_str(url_str).path_parts();
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

        Some(box Resource {
            window: Window::new(p, s, title),
            seek: 0,
        })
    }

    pub fn event(&mut self, event: &Event) {
        self.session.event(event);
        unsafe { self.session.redraw() };
    }
}

//TODO: This is a hack and it will go away
#[cold]
#[inline(never)]
#[no_mangle]
pub unsafe extern "C" fn _event(scheme: *mut Scheme, event: *const Event) {
    (*scheme).event(&*event);
}
