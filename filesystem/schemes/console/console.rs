use redox::Box;
use redox::console::ConsoleWindow;
use redox::fs::file::File;
use redox::str;
use redox::string::*;
use redox::io::{Read, Write, Seek, SeekFrom};

pub struct Resource {
    console_window: Box<ConsoleWindow>,
    line_end_toggle: bool,
}

impl Resource {
    pub fn dup(&self) -> Option<Box<Resource>> {
        Some(box Resource {
            console_window: ConsoleWindow::new(100, 100, 640, 480, &self.console_window.window.title()),
            line_end_toggle: false
        })
    }

    pub fn path(&self, buf: &mut [u8]) -> Option<usize> {
        let path = "console:".to_string() + &self.console_window.window.title();

        let mut i = 0;
        for b in path.bytes() {
            if i < buf.len() {
                buf[i] = b;
                i += 1;
            } else {
                break;
            }
        }

        Some(i)
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        if self.line_end_toggle {
            self.line_end_toggle = false;
            Some(0)
        } else {
            match self.console_window.read() {
                Some(string) => {
                    self.line_end_toggle = true;

                    let mut i = 0;

                    for b in string.bytes() {
                        if i < buf.len() {
                            buf[i] = b;
                            i += 1;
                        }else{
                            break;
                        }
                    }

                    Some(i)
                },
                None => None
            }
        }
    }

    pub fn write(&mut self, buf: &[u8]) -> Option<usize> {
        self.console_window.print(unsafe { &str::from_utf8_unchecked(buf) }, [224, 224, 224, 256]);
        self.sync();

        Some(buf.len())
    }

    pub fn seek(&mut self, seek: SeekFrom) -> Option<usize> {
        None
    }

    pub fn sync(&mut self) -> bool {
        self.console_window.sync();
        true
    }
}

pub struct Scheme;

impl Scheme {
    pub fn new() -> Box<Self> {
        box Scheme
    }

    pub fn open(&mut self, path: &str) -> Option<Box<Resource>> {
        let (scheme, mut title) = path.split_at(path.find(':').unwrap_or(path.len() - 1) + 1);

        if title.len() == 0 {
            title = "Console";
        }

        Some(box Resource {
            console_window: ConsoleWindow::new(100, 100, 640, 480, title),
            line_end_toggle: false
        })
    }
}
