use redox::{Box, String, Url};
use redox::{cmp, mem, ptr};
use redox::fs::File;
use redox::get_slice::GetSlice;
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
            window: Window::new(self.window.point,
                                self.window.size,
                                self.window.title.clone()),
            seek: self.seek,
        })
    }

    /// Return the url of this resource
    pub fn path(&self) -> Option<String> {
        Some(format!("orbital:///{}/{}/{}/{}/{}",
                     self.window.point.x,
                     self.window.point.y,
                     self.window.size.width,
                     self.window.size.height,
                     self.window.title))
    }

    /// Read data to buffer
    pub fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        // Read events from window
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
            Display::copy_run(buf.as_ptr() as usize, content.offscreen + self.seek, size);
        }
        self.seek += size;

        return Some(size);
    }

    /// Seek
    pub fn seek(&mut self, pos: SeekFrom) -> Option<usize> {
        let end = self.window.content.size;

        self.seek = match pos {
            SeekFrom::Start(offset) => cmp::min(end, cmp::max(0, offset)),
            SeekFrom::Current(offset) =>
                cmp::min(end, cmp::max(0, self.seek as isize + offset) as usize),
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

/// A window scheme
pub struct Scheme {
    pub session: Box<Session>,
    pub next_x: isize,
    pub next_y: isize,
}

impl Scheme {
    pub fn new() -> Box<Scheme> {
        debugln!("- Starting Orbital");
        debugln!("    Console: Press F1");
        debugln!("    Desktop: Press F2");
        let mut ret = box Scheme {
            session: Session::new(),
            next_x: 0,
            next_y: 0,
        };
        unsafe { session_ptr = ret.session.deref_mut() };
        ret
    }

    pub fn open(&mut self, url_str: &str, _: usize) -> Option<Box<Resource>> {
        // window://host/path/path/path is the path type we're working with.
        let url = Url::from_str(url_str);

        let host = url.host();
        if host.is_empty() {
            let path = url.path_parts();
            let mut pointx = match path.get(0) {
                Some(x) => x.to_num_signed(),
                None => 0,
            };
            let mut pointy = match path.get(1) {
                Some(y) => y.to_num_signed(),
                None => 0,
            };
            let size_width = match path.get(2) {
                Some(w) => w.to_num(),
                None => 100,
            };
            let size_height = match path.get(3) {
                Some(h) => h.to_num(),
                None => 100,
            };

            let mut title = match path.get(4) {
                Some(t) => t.clone(),
                None => String::new(),
            };
            for i in 5..path.len() {
                if let Some(t) = path.get(i) {
                    title = title + "/" + t;
                }
            }

            if pointx <= 0 || pointy <= 0 {
                if self.next_x > self.session.display.width as isize - size_width as isize {
                    self.next_x = 0;
                }
                self.next_x += 32;
                pointx = self.next_x;

                if self.next_y > self.session.display.height as isize - size_height as isize {
                    self.next_y = 0;
                }
                self.next_y += 32;
                pointy = self.next_y;
            }

            Some(box Resource {
                window: Window::new(Point::new(pointx, pointy),
                                    Size::new(size_width, size_height),
                                    title),
                seek: 0,
            })
        } else if host == "launch" {
            let path = url.path();

            unsafe {
                let reenable = scheduler::start_no_ints();

                for package in self.session.packages.iter() {
                    let mut accepted = false;
                    for accept in package.accepts.iter() {
                        if (accept.starts_with('*') &&
                            path.ends_with(&accept.get_slice(Some(1), None))) ||
                           (accept.ends_with('*') &&
                            path.starts_with(&accept.get_slice(None, Some(accept.len() - 1)))) {
                            accepted = true;
                            break;
                        }
                    }
                    if accepted {
                        File::exec(&package.binary, &[&path]);
                        break;
                    }
                }

                scheduler::end_no_ints(reenable);
            }

            None
        } else {
            None
        }
    }

    pub fn event(&mut self, event: &Event) {
        unsafe {
            let reenable = scheduler::start_no_ints();

            self.session.event(event);

            scheduler::end_no_ints(reenable);

            self.session.redraw();
        }
    }
}

// TODO: This is a hack and it will go away
#[cold]
#[inline(never)]
#[no_mangle]
pub unsafe extern "C" fn _event(scheme: *mut Scheme, event: *const Event) {
    (*scheme).event(&*event);
}
