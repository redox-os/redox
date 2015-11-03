use alloc::boxed::Box;

use collections::string::ToString;

use core::{cmp, mem, ptr};

use graphics::display::Display;

use schemes::{KScheme, Resource, ResourceSeek, URL};

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
    fn url(&self) -> URL {
        return URL::from_string(&("display://".to_string()));
    }

    // get display info here
    // currently only the dimensions
    fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        unsafe {
            if mem::size_of::<[usize;2]>() <= buf.len() {
                let size: [usize; 2] = [ self.display.width, self.display.height ];
                unsafe { ptr::write(buf.as_ptr() as *mut [usize; 2], size); }
                Some(mem::size_of::<[usize;2]>())
            } else {
                None
            }
        }
    }

    fn write(&mut self, buf: &[u8]) -> Option<usize> {
        let display = &mut self.display;

        let size = cmp::min(display.size - self.seek, buf.len());
        unsafe {
            Display::copy_run(buf.as_ptr() as usize,
                              display.offscreen + self.seek,
                              size);
        }
        self.seek += size;
        return Some(size);
    }

    fn seek(&mut self, pos: ResourceSeek) -> Option<usize> {
        let end = self.display.size;

        self.seek = match pos {
            ResourceSeek::Start(offset) => cmp::min(end, cmp::max(0, offset)),
            ResourceSeek::Current(offset) => cmp::min(end, cmp::max(0, self.seek as isize + offset) as usize),
            ResourceSeek::End(offset) => cmp::min(end, cmp::max(0, end as isize + offset) as usize),
        };

        return Some(self.seek);
    }

    fn sync(&mut self) -> bool {
        self.display.flip();
        true
    }
}

impl KScheme for DisplayScheme {
    fn scheme(&self) -> &str {
        "display"
    }

    fn open(&mut self, _: &URL) -> Option<Box<Resource>> {
        // TODO: ponder these things:
        // - should display:// be the only only valid url
        //      for this scheme?
        // - maybe "read" should support displays at some other location
        //      like built in screen sharing capability or something
        unsafe {
            return Some(box DisplayResource {
                        display: Display::root(),
                       seek: 0,
            });
        }
    }
}
