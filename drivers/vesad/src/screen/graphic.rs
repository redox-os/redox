use std::collections::VecDeque;
use std::{cmp, mem, slice};

use orbclient::{Event, EventOption};
use syscall::error::*;
use syscall::flag::{SEEK_SET, SEEK_CUR, SEEK_END};

use display::Display;
use primitive::fast_copy;
use screen::Screen;

pub struct GraphicScreen {
    pub display: Display,
    pub seek: usize,
    pub mouse_x: i32,
    pub mouse_y: i32,
    pub input: VecDeque<Event>,
    pub requested: usize
}

impl GraphicScreen {
    pub fn new(display: Display) -> GraphicScreen {
        GraphicScreen {
            display: display,
            seek: 0,
            mouse_x: 0,
            mouse_y: 0,
            input: VecDeque::new(),
            requested: 0
        }
    }
}

impl Screen for GraphicScreen {
    fn width(&self) -> usize {
        self.display.width
    }

    fn height(&self) -> usize {
        self.display.height
    }

    fn event(&mut self, flags: usize) -> Result<usize> {
        self.requested = flags;
        Ok(0)
    }

    fn map(&self, offset: usize, size: usize) -> Result<usize> {
        if offset + size <= self.display.offscreen.len() * 4 {
            Ok(self.display.offscreen.as_ptr() as usize + offset)
        } else {
            Err(Error::new(EINVAL))
        }
    }

    fn input(&mut self, event: &Event) {
        if let EventOption::Mouse(mut mouse_event) = event.to_option() {
            let x = cmp::max(0, cmp::min(self.display.width as i32, self.mouse_x + mouse_event.x));
            let y = cmp::max(0, cmp::min(self.display.height as i32, self.mouse_y + mouse_event.y));

            mouse_event.x = x;
            self.mouse_x = x;
            mouse_event.y = y;
            self.mouse_y = y;

            self.input.push_back(mouse_event.to_event());
        } else {
            self.input.push_back(*event);
        }
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut i = 0;

        let event_buf = unsafe { slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut Event, buf.len()/mem::size_of::<Event>()) };

        while i < event_buf.len() && ! self.input.is_empty() {
            event_buf[i] = self.input.pop_front().unwrap();
            i += 1;
        }

        Ok(i * mem::size_of::<Event>())
    }

    fn will_block(&self) -> bool {
        self.input.is_empty()
    }

    fn write(&mut self, buf: &[u8], sync: bool) -> Result<usize> {
        let size = cmp::max(0, cmp::min(self.display.offscreen.len() as isize - self.seek as isize, (buf.len()/4) as isize)) as usize;

        if size > 0 {
            unsafe {
                fast_copy(self.display.offscreen.as_mut_ptr().offset(self.seek as isize) as *mut u8, buf.as_ptr(), size * 4);
                if sync {
                    fast_copy(self.display.onscreen.as_mut_ptr().offset(self.seek as isize) as *mut u8, buf.as_ptr(), size * 4);
                }
            }
        }

        Ok(size * 4)
    }

    fn seek(&mut self, pos: usize, whence: usize) -> Result<usize> {
        let size = self.display.offscreen.len();

        self.seek = match whence {
            SEEK_SET => cmp::min(size, (pos/4)),
            SEEK_CUR => cmp::max(0, cmp::min(size as isize, self.seek as isize + (pos/4) as isize)) as usize,
            SEEK_END => cmp::max(0, cmp::min(size as isize, size as isize + (pos/4) as isize)) as usize,
            _ => return Err(Error::new(EINVAL))
        };

        Ok(self.seek * 4)
    }

    fn sync(&mut self) {
        self.redraw();
    }

    fn redraw(&mut self) {
        let width = self.display.width;
        let height = self.display.height;
        self.display.sync(0, 0, width, height);
    }
}
