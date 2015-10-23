use redox::Box;
use redox::cell::UnsafeCell;
use redox::Color;
use redox::console::ConsoleWindow;
use redox::rc::Rc;
use redox::str;
use redox::string::*;
use redox::io::SeekFrom;

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

    pub fn dup(&self) -> Option<Box<Self>> {
        Some(box Resource {
            console_window: self.console_window.clone(),
            line_end_toggle: false
        })
    }

    pub fn path(&self) -> Option<String> {
        Some("console:".to_string() + &self.inner().window.title())
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        if self.line_end_toggle {
            self.line_end_toggle = false;
            Some(0)
        } else {
            match self.inner_mut().read() {
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
        self.inner_mut().print(unsafe { &str::from_utf8_unchecked(buf) }, Color::rgba(224, 224, 224, 255));
        self.sync();

        Some(buf.len())
    }

    pub fn seek(&mut self, seek: SeekFrom) -> Option<usize> {
        None
    }

    pub fn sync(&mut self) -> bool {
        self.inner_mut().sync();
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
            console_window: Rc::new(UnsafeCell::new(ConsoleWindow::new(100, 100, 640, 480, title))),
            line_end_toggle: false
        })
    }
}
