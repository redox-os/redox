use alloc::boxed::Box;

use graphics::display::*;

use core::cmp::{min, max};

use common::resource::*;
use common::string::*;

use programs::session::SessionItem;

pub struct DisplayScheme {
	// this is definitely not what needs to be done
	// at least not here
	pub allocated: bool,
}

// Should there only be one display per session?
pub struct DisplayResource {
	pub display: Box<Display>,
	pub seek: usize,
}

impl Resource for DisplayResource {
	// can't think of when you would wish to duplicate a display
	fn dup(&self) -> Option<Box<Resource>> {
		None
	}

	/// Return the URL for display resource
	fn url(&self) -> URL {
		return URL::from_string(&("display://".to_string()));
	}

	// not sure what to return here
	fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
		None
	}

	
	fn write(&mut self, buf: &[u8]) -> Option<usize> {
		let display = &mut self.display;

		let size = min(display.size - self.seek, buf.len());
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
			ResourceSeek::Start(offset) => min(end, max(0, offset)),
			ResourceSeek::Current(offset) => min(end, max(0, self.seek as isize + offset) as usize),
			ResourceSeek::End(offset) => min(end, max(0, end as isize + offset) as usize),
		};

		return Some(self.seek);
	}

	fn sync(&mut self) -> bool {
		self.display.flip();
		return true;
	}
}

impl SessionItem for DisplayScheme {
	fn scheme(&self) -> String {
		return "display".to_string();
	}

	fn open(&mut self, url: &URL) -> Option<Box<Resource>> {
		// only valid url should be display://
		// at least for now, maybe it would be useful to
		// connect to another display to read it (screen sharing)
		// maybe write to it too, for the express purpose of harassing somebody
		// MUAHAHAHA
		if !self.allocated {
			self.allocated = true;
			unsafe {
				return Some(box DisplayResource {
					display: Display::root(),
					seek: 0,
				});
			}
		} 
		return None;
	}
}
