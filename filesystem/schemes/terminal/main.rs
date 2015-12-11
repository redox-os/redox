use std::Box;
use std::cell::UnsafeCell;
use std::io::{Result, SeekFrom};
use std::rc::Rc;
use std::str;
use std::string::{String, ToString};
use std::syscall::SysError;
use std::syscall::{EINVAL, ESPIPE};

use orbital::Color;
use self::window::ConsoleWindow;

pub mod window;

pub struct Resource {
    console_window: Rc<UnsafeCell<Box<ConsoleWindow>>>,
    line_end_toggle: bool,
}

impl Resource {
    fn inner(&self) -> &Box<ConsoleWindow> {
        unsafe { &*self.console_window.get() }
    }

    fn inner_mut(&mut self) -> &mut Box<ConsoleWindow> {
        unsafe { &mut *self.console_window.get() }
    }

    pub fn dup(&self) -> Result<Box<Self>> {
        Ok(box Resource {
            console_window: self.console_window.clone(),
            line_end_toggle: false,
        })
    }

    pub fn path(&self) -> Result<String> {
        Ok("terminal:".to_string() + &self.inner().window.title())
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if self.line_end_toggle {
            self.line_end_toggle = false;
            Ok(0)
        } else {
            match self.inner_mut().read() {
                Some(string) => {
                    self.line_end_toggle = true;

                    let mut i = 0;

                    for b in string.bytes() {
                        if i < buf.len() {
                            buf[i] = b;
                            i += 1;
                        } else {
                            break;
                        }
                    }

                    Ok(i)
                }
                None => Err(SysError::new(EINVAL)),
            }
        }
    }

    pub fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.inner_mut().print(unsafe { &str::from_utf8_unchecked(buf) },
                               Color::rgba(224, 224, 224, 255));
        self.sync();

        Ok(buf.len())
    }

    pub fn seek(&mut self, _: SeekFrom) -> Result<usize> {
        Err(SysError::new(ESPIPE))
    }

    pub fn sync(&mut self) -> Result<()> {
        self.inner_mut().sync();
        Ok(())
    }
}

pub struct Scheme;

impl Scheme {
    pub fn new() -> Box<Self> {
        box Scheme
    }

    pub fn open(&mut self, path: &str, _: usize) -> Result<Box<Resource>> {
        let (_, title) = path.split_at(path.find(':').unwrap_or(path.len() - 1) + 1);

        Ok(box Resource {
            console_window: Rc::new(UnsafeCell::new(ConsoleWindow::new(-1, -1, 640, 480, title))),
            line_end_toggle: false,
        })
    }
}
