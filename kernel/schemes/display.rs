use alloc::boxed::Box;

use common::event::Event;

use core::{cmp, ptr};
use core::mem::size_of;

use graphics::display::Display;

use schemes::{Result, KScheme, Resource, ResourceSeek, Url};

use system::error::{Error, ENOENT, EINVAL};

pub struct DisplayScheme;

// Should there only be one display per session?
/// A display resource
pub struct DisplayResource {
    /// The display
    pub display: Box<Display>,
    /// Seek
    pub seek: usize,
}

impl Resource for DisplayResource {
    /// Return the URL for display resource
    fn url(&self) -> Url {
        Url::from_string(format!("display:{}/{}", self.display.width, self.display.height))
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if buf.len() >= size_of::<Event>() {
            let mut i = 0;
            if ! ::env().console.lock().draw {
                while i <= buf.len() - size_of::<Event>() {
                    if let Some(event) = ::env().events.lock().pop_front() {
                        unsafe { ptr::write(buf.as_mut_ptr().offset(i as isize) as *mut Event, event) };
                        i += size_of::<Event>();
                    } else {
                        break;
                    }
                }
            }
            Ok(i)
        } else {
            Err(Error::new(EINVAL))
        }
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        if ! ::env().console.lock().draw {
            let size = cmp::max(0, cmp::min(self.display.size as isize - self.seek as isize, buf.len() as isize)) as usize;

            if size > 0 {
                unsafe {
                    Display::copy_run(buf.as_ptr() as usize, self.display.onscreen + self.seek, size);
                }
            }

            Ok(size)
        } else {
            Ok(0)
        }
    }

    fn seek(&mut self, pos: ResourceSeek) -> Result<usize> {
        self.seek = match pos {
            ResourceSeek::Start(offset) => cmp::min(self.display.size, cmp::max(0, offset)),
            ResourceSeek::Current(offset) => cmp::min(self.display.size, cmp::max(0, self.seek as isize + offset) as usize),
            ResourceSeek::End(offset) => cmp::min(self.display.size, cmp::max(0, self.display.size as isize + offset) as usize),
        };

        Ok(self.seek)
    }

    fn sync(&mut self) -> Result<()> {
        Ok(())
    }
}

impl KScheme for DisplayScheme {
    fn scheme(&self) -> &str {
        "display"
    }

    fn open(&mut self, _: &Url, _: usize) -> Result<Box<Resource>> {
        if let Some(display) = unsafe { Display::root() } {
            ::env().console.lock().draw = false;
            Ok(box DisplayResource {
                display: display,
                seek: 0,
            })
        } else {
            Err(Error::new(ENOENT))
        }
    }
}
