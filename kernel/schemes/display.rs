use alloc::boxed::Box;

use collections::string::ToString;

use core::cmp;

use graphics::display::Display;

use schemes::{Result, KScheme, Resource, ResourceSeek, Url};

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
        Url::from_string("display:".to_string())
    }


    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let display = &mut self.display;

        let size = cmp::min(display.size - self.seek, buf.len());
        unsafe {
            Display::copy_run(buf.as_ptr() as usize, display.offscreen + self.seek, size);
        }
        self.seek += size;
        return Ok(size);
    }

    fn seek(&mut self, pos: ResourceSeek) -> Result<usize> {
        let end = self.display.size;

        self.seek = match pos {
            ResourceSeek::Start(offset) => cmp::min(end, cmp::max(0, offset)),
            ResourceSeek::Current(offset) =>
                cmp::min(end, cmp::max(0, self.seek as isize + offset) as usize),
            ResourceSeek::End(offset) => cmp::min(end, cmp::max(0, end as isize + offset) as usize),
        };

        return Ok(self.seek);
    }

    fn sync(&mut self) -> Result<()> {
        self.display.flip();
        Ok(())
    }
}

impl KScheme for DisplayScheme {
    fn scheme(&self) -> &str {
        "display"
    }

    fn open(&mut self, _: &Url, _: usize) -> Result<Box<Resource>> {
        // TODO: ponder these things:
        // - should display: be the only only valid url
        //      for this scheme?
        // - maybe "read" should support displays at some other location
        //      like built in screen sharing capability or something
        unsafe {
            return Ok(box DisplayResource {
                display: Display::root(),
                seek: 0,
            });
        }
    }
}
