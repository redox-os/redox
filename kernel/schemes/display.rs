use alloc::boxed::Box;

use core::cmp;

use graphics::display::Display;

use schemes::{Result, KScheme, Resource, ResourceSeek, Url};

use system::error::{Error, ENOENT};

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

impl KScheme for DisplayScheme {
    fn scheme(&self) -> &str {
        "display"
    }

    fn open(&mut self, _: &Url, _: usize) -> Result<Box<Resource>> {
        if let Some(display) = unsafe { Display::root() } {
            Ok(box DisplayResource {
                display: display,
                seek: 0,
            })
        } else {
            Err(Error::new(ENOENT))
        }
    }
}
