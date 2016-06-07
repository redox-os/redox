use alloc::boxed::Box;

use collections::String;

use common::event::Event;

use core::{cmp, ptr};
use core::mem::size_of;

use fs::{KScheme, Resource, ResourceSeek, Url};

use system::error::{EACCES, EBADF, EINVAL, ENOENT, Error, Result};
use system::graphics::fast_copy;

/// A display resource
pub struct DisplayResource {
    /// Path
    path: String,
    /// Seek
    seek: usize,
}

impl Resource for DisplayResource {
    fn dup(&self) -> Result<Box<Resource>> {
        Ok(Box::new(DisplayResource {
            path: self.path.clone(),
            seek: self.seek,
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
        let console = ::env().console.lock();
        if let Some(ref display) = console.display {
            let size = cmp::max(0,
                                cmp::min(display.size as isize - self.seek as isize,
                                         (buf.len() / 4) as isize)) as usize;

            if size > 0 {
                unsafe {
                    fast_copy(display.onscreen.offset(self.seek as isize),
                              buf.as_ptr() as *const u32,
                              size);
                }
            }

            Ok(size)
        } else {
            Err(Error::new(EBADF))
        }
    }

    fn seek(&mut self, pos: ResourceSeek) -> Result<usize> {
        let console = ::env().console.lock();
        if let Some(ref display) = console.display {
            self.seek = match pos {
                ResourceSeek::Start(offset) => cmp::min(display.size, cmp::max(0, offset)),
                ResourceSeek::Current(offset) => {
                    cmp::min(display.size,
                             cmp::max(0, self.seek as isize + offset) as usize)
                },
                ResourceSeek::End(offset) => {
                    cmp::min(display.size,
                             cmp::max(0, display.size as isize + offset) as usize)
                },
            };

            Ok(self.seek)
        } else {
            Err(Error::new(EBADF))
        }
    }

    fn sync(&mut self) -> Result<()> {
        Ok(())
    }
}

pub struct DisplayScheme;

impl KScheme for DisplayScheme {
    fn scheme(&self) -> &str {
        "display"
    }

    fn open(&mut self, url: Url, _: usize) -> Result<Box<Resource>> {
        if url.reference() == "manager" {
            let mut console = ::env().console.lock();
            if console.draw {
                console.draw = false;

                if let Some(ref display) = console.display {
                    Ok(box DisplayResource {
                        path: format!("display:{}/{}", display.width, display.height),
                        seek: 0,
                    })
                } else {
                    Err(Error::new(ENOENT))
                }
            } else {
                Err(Error::new(EACCES))
            }
        } else {
            let console = ::env().console.lock();
            if let Some(ref display) = console.display {
                Ok(box DisplayResource {
                    path: format!("display:{}/{}", display.width, display.height),
                    seek: 0,
                })
            } else {
                Err(Error::new(ENOENT))
            }
        }
    }
}
