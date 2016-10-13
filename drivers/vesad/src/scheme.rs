use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::{mem, slice, str};

use orbclient::{Event, EventOption};
use syscall::{Result, Error, EACCES, EBADF, ENOENT, Scheme};

use display::Display;
use screen::{Screen, GraphicScreen, TextScreen};

pub struct DisplayScheme {
    width: usize,
    height: usize,
    onscreen: usize,
    active: Cell<usize>,
    screens: RefCell<BTreeMap<usize, Box<Screen>>>
}

impl DisplayScheme {
    pub fn new(width: usize, height: usize, onscreen: usize) -> DisplayScheme {
        let mut screens: BTreeMap<usize, Box<Screen>> = BTreeMap::new();
        screens.insert(1, Box::new(TextScreen::new(Display::new(width, height, onscreen))));
        screens.insert(2, Box::new(GraphicScreen::new(Display::new(width, height, onscreen))));

        DisplayScheme {
            width: width,
            height: height,
            onscreen: onscreen,
            active: Cell::new(1),
            screens: RefCell::new(screens)
        }
    }

    pub fn will_block(&self, id: usize) -> bool {
        let screens = self.screens.borrow();
        if let Some(screen) = screens.get(&id) {
            screen.will_block()
        } else {
            false
        }
    }
}

impl Scheme for DisplayScheme {
    fn open(&self, path: &[u8], _flags: usize, uid: u32, _gid: u32) -> Result<usize> {
        if path == b"input" {
            if uid == 0 {
                Ok(0)
            } else {
                Err(Error::new(EACCES))
            }
        } else {
            let path_str = str::from_utf8(path).unwrap_or("");
            let id = path_str.parse::<usize>().unwrap_or(0);
            if self.screens.borrow().contains_key(&id) {
                Ok(id)
            } else {
                Err(Error::new(ENOENT))
            }
        }
    }

    fn dup(&self, id: usize) -> Result<usize> {
        Ok(id)
    }

    fn fevent(&self, id: usize, flags: usize) -> Result<usize> {
        let mut screens = self.screens.borrow_mut();
        if let Some(mut screen) = screens.get_mut(&id) {
            screen.event(flags)
        } else {
            Err(Error::new(EBADF))
        }
    }

    fn fpath(&self, id: usize, buf: &mut [u8]) -> Result<usize> {
        let screens = self.screens.borrow();
        let path_str = if id == 0 {
            format!("display:input")
        } else if let Some(screen) = screens.get(&id) {
            format!("display:{}/{}/{}", id, screen.width(), screen.height())
        } else {
            return Err(Error::new(EBADF));
        };

        let path = path_str.as_bytes();

        let mut i = 0;
        while i < buf.len() && i < path.len() {
            buf[i] = path[i];
            i += 1;
        }

        Ok(i)
    }

    fn fsync(&self, id: usize) -> Result<usize> {
        let mut screens = self.screens.borrow_mut();
        if let Some(mut screen) = screens.get_mut(&id) {
            if id == self.active.get() {
                screen.sync();
            }
            Ok(0)
        } else {
            Err(Error::new(EBADF))
        }
    }

    fn read(&self, id: usize, buf: &mut [u8]) -> Result<usize> {
        let mut screens = self.screens.borrow_mut();
        if let Some(mut screen) = screens.get_mut(&id) {
            screen.read(buf)
        } else {
            Err(Error::new(EBADF))
        }
    }

    fn write(&self, id: usize, buf: &[u8]) -> Result<usize> {
        let mut screens = self.screens.borrow_mut();
        if id == 0 {
            if buf.len() == 1 && buf[0] >= 0xF4 {
                let new_active = (buf[0] - 0xF4) as usize + 1;
                if let Some(mut screen) = screens.get_mut(&new_active) {
                    self.active.set(new_active);
                    screen.redraw();
                }
                Ok(1)
            } else {
                let events = unsafe { slice::from_raw_parts(buf.as_ptr() as *const Event, buf.len()/mem::size_of::<Event>()) };

                for event in events.iter() {
                    let new_active_opt = if let EventOption::Key(key_event) = event.to_option() {
                        match key_event.scancode {
                            f @ 0x3B ... 0x44 => { // F1 through F10
                                Some((f - 0x3A) as usize)
                            },
                            0x57 => { // F11
                                Some(11)
                            },
                            0x58 => { // F12
                                Some(12)
                            },
                            _ => None
                        }
                    } else {
                        None
                    };

                    if let Some(new_active) = new_active_opt {
                        if let Some(mut screen) = screens.get_mut(&new_active) {
                            self.active.set(new_active);
                            screen.redraw();
                        }
                    } else {
                        if let Some(mut screen) = screens.get_mut(&self.active.get()) {
                            screen.input(event);
                        }
                    }
                }

                Ok(events.len() * mem::size_of::<Event>())
            }
        } else if let Some(mut screen) = screens.get_mut(&id) {
            screen.write(buf, id == self.active.get())
        } else {
            Err(Error::new(EBADF))
        }
    }

    fn seek(&self, id: usize, pos: usize, whence: usize) -> Result<usize> {
        let mut screens = self.screens.borrow_mut();
        if let Some(mut screen) = screens.get_mut(&id) {
            screen.seek(pos, whence)
        } else {
            Err(Error::new(EBADF))
        }
    }

    fn close(&self, _id: usize) -> Result<usize> {
        Ok(0)
    }
}
