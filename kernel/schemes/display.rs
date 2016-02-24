use alloc::arc::Arc;
use alloc::boxed::Box;

use collections::String;

use common::event::Event;

use core::{cmp, ptr};
use core::mem::size_of;

use graphics::display::Display;

use fs::{KScheme, Resource, ResourceSeek, Url};

use system::error::{Error, Result, EACCES, ENOENT, EINVAL};

pub struct DisplayScheme;

// Should there only be one display per session?
/// A display resource
pub struct DisplayResource {
    /// Path
    path: String,
    /// The display
    display: Arc<Box<Display>>,
    /// Seek
    seek: usize,
}

impl Resource for DisplayResource {
    fn dup(&self) -> Result<Box<Resource>> {
        Ok(Box::new(DisplayResource {
            path: self.path.clone(),
            display: self.display.clone(),
            seek: self.seek
        }))
    }

    /// Return the URL for display resource
    fn path(&self, buf: &mut [u8]) -> Result<usize> {
        let path = self.path.as_bytes();

        for (b, p) in buf.iter_mut().zip(path.iter()) {
            *b = *p;
        }

        Ok(cmp::min(buf.len(), path.len()))
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if buf.len() >= size_of::<Event>() {
            let event = ::env().events.receive();
            unsafe { ptr::write(buf.as_mut_ptr().offset(0isize) as *mut Event, event) };
            let mut i = size_of::<Event>();

            while i + size_of::<Event>() <= buf.len() {
                if let Some(event) = ::env().events.inner.lock().pop_front() {
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

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let size = cmp::max(0, cmp::min(self.display.size as isize - self.seek as isize, buf.len() as isize)) as usize;

        if size > 0 {
            unsafe {
                Display::copy_run(buf.as_ptr() as usize, self.display.onscreen + self.seek, size);
            }
        }

        Ok(size)
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

impl Drop for DisplayScheme {
    fn drop(&mut self){
        ::env().console.lock().draw = true;
    }
}

impl KScheme for DisplayScheme {
    fn scheme(&self) -> &str {
        "display"
    }

    fn open(&mut self, _: &Url, _: usize) -> Result<Box<Resource>> {
        if ::env().console.lock().draw {
            if let Some(display) = Display::root() {
                ::env().console.lock().draw = false;

                Ok(box DisplayResource {
                    path: format!("display:{}/{}", display.width, display.height),
                    display: Arc::new(display),
                    seek: 0,
                })
            } else {
                Err(Error::new(ENOENT))
            }
        } else {
            Err(Error::new(EACCES))
        }
    }
}
