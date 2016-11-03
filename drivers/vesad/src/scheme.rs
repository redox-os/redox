use std::collections::BTreeMap;
use std::{mem, slice, str};

use orbclient::{Event, EventOption};
use syscall::{Result, Error, EACCES, EBADF, ENOENT, SchemeMut};

use display::Display;
use screen::{Screen, GraphicScreen, TextScreen};

pub struct DisplayScheme {
    active: usize,
    pub screens: BTreeMap<usize, Box<Screen>>
}

impl DisplayScheme {
    pub fn new(width: usize, height: usize, onscreen: usize, spec: &[bool]) -> DisplayScheme {
        let mut screens: BTreeMap<usize, Box<Screen>> = BTreeMap::new();

        let mut screen_i = 1;
        for &screen_type in spec.iter() {
            if screen_type {
                screens.insert(screen_i, Box::new(GraphicScreen::new(Display::new(width, height, onscreen))));
            } else {
                screens.insert(screen_i, Box::new(TextScreen::new(Display::new(width, height, onscreen))));
            }
            screen_i += 1;
        }

        DisplayScheme {
            active: 1,
            screens: screens
        }
    }

    pub fn will_block(&self, id: usize) -> bool {
        if let Some(screen) = self.screens.get(&id) {
            screen.will_block()
        } else {
            false
        }
    }
}

impl SchemeMut for DisplayScheme {
    fn open(&mut self, path: &[u8], _flags: usize, uid: u32, _gid: u32) -> Result<usize> {
        if path == b"input" {
            if uid == 0 {
                Ok(0)
            } else {
                Err(Error::new(EACCES))
            }
        } else {
            let path_str = str::from_utf8(path).unwrap_or("");
            let id = path_str.parse::<usize>().unwrap_or(0);
            if self.screens.contains_key(&id) {
                Ok(id)
            } else {
                Err(Error::new(ENOENT))
            }
        }
    }

    fn dup(&mut self, id: usize, _buf: &[u8]) -> Result<usize> {
        Ok(id)
    }

    fn fevent(&mut self, id: usize, flags: usize) -> Result<usize> {
        if let Some(mut screen) = self.screens.get_mut(&id) {
            screen.event(flags).and(Ok(id))
        } else {
            Err(Error::new(EBADF))
        }
    }

    fn fmap(&mut self, id: usize, offset: usize, size: usize) -> Result<usize> {
        if let Some(screen) = self.screens.get(&id) {
            screen.map(offset, size)
        } else {
            Err(Error::new(EBADF))
        }
    }

    fn fpath(&mut self, id: usize, buf: &mut [u8]) -> Result<usize> {
        let path_str = if id == 0 {
            format!("display:input")
        } else if let Some(screen) = self.screens.get(&id) {
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

    fn fsync(&mut self, id: usize) -> Result<usize> {
        if let Some(mut screen) = self.screens.get_mut(&id) {
            if id == self.active {
                screen.sync();
            }
            Ok(0)
        } else {
            Err(Error::new(EBADF))
        }
    }

    fn read(&mut self, id: usize, buf: &mut [u8]) -> Result<usize> {
        if let Some(mut screen) = self.screens.get_mut(&id) {
            screen.read(buf)
        } else {
            Err(Error::new(EBADF))
        }
    }

    fn write(&mut self, id: usize, buf: &[u8]) -> Result<usize> {
        if id == 0 {
            if buf.len() == 1 && buf[0] >= 0xF4 {
                let new_active = (buf[0] - 0xF4) as usize + 1;
                if let Some(mut screen) = self.screens.get_mut(&new_active) {
                    self.active = new_active;
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
                        if let Some(mut screen) = self.screens.get_mut(&new_active) {
                            self.active = new_active;
                            screen.redraw();
                        }
                    } else {
                        if let Some(mut screen) = self.screens.get_mut(&self.active) {
                            screen.input(event);
                        }
                    }
                }

                Ok(events.len() * mem::size_of::<Event>())
            }
        } else if let Some(mut screen) = self.screens.get_mut(&id) {
            screen.write(buf, id == self.active)
        } else {
            Err(Error::new(EBADF))
        }
    }

    fn seek(&mut self, id: usize, pos: usize, whence: usize) -> Result<usize> {
        if let Some(mut screen) = self.screens.get_mut(&id) {
            screen.seek(pos, whence)
        } else {
            Err(Error::new(EBADF))
        }
    }

    fn close(&mut self, _id: usize) -> Result<usize> {
        Ok(0)
    }
}
