use alloc::arc::{Arc, Weak};
use alloc::boxed::Box;

use collections::vec_deque::VecDeque;

use fs::Resource;

use arch::context::context_switch;
use arch::intex::Intex;

use system::error::{Error, Result, EPIPE};

/// Read side of a pipe
pub struct PipeRead {
    vec: Arc<Intex<VecDeque<u8>>>,
    eof_toggle: bool,
}

impl PipeRead {
    pub fn new() -> Self {
        PipeRead {
            vec: Arc::new(Intex::new(VecDeque::new())),
            eof_toggle: false,
        }
    }
}

impl Resource for PipeRead {
    fn dup(&self) -> Result<Box<Resource>> {
        Ok(box PipeRead {
            vec: self.vec.clone(),
            eof_toggle: self.eof_toggle,
        })
    }

    fn path(&self, buf: &mut [u8]) -> Result <usize> {
        let path = b"pipe:r";

        let mut i = 0;
        while i < buf.len() && i < path.len() {
            buf[i] = path[i];
            i += 1;
        }

        Ok(i)
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if self.eof_toggle {
            self.eof_toggle = false;
            return Ok(0);
        }

        loop {
            {
                let mut vec = self.vec.lock();
                if vec.is_empty() {
                    if Arc::weak_count(&self.vec) == 0 {
                        return Ok(0);
                    }
                } else {
                    let mut i = 0;
                    while i < buf.len() {
                        match vec.pop_front() {
                            Some(b) => {
                                buf[i] = b;
                                i += 1;
                            },
                            None => break
                        }
                    }
                    self.eof_toggle = true;
                    return Ok(i);
                }
            }

            unsafe { context_switch() };
        }
    }
}

/// Read side of a pipe
pub struct PipeWrite {
    vec: Weak<Intex<VecDeque<u8>>>,
}

impl PipeWrite {
    pub fn new(read: &PipeRead) -> Self {
        PipeWrite {
            vec: Arc::downgrade(&read.vec),
        }
    }
}

impl Resource for PipeWrite {
    fn dup(&self) -> Result<Box<Resource>> {
        Ok(box PipeWrite {
            vec: self.vec.clone(),
        })
    }

    fn path(&self, buf: &mut [u8]) -> Result <usize> {
        let path = b"pipe:w";

        let mut i = 0;
        while i < buf.len() && i < path.len() {
            buf[i] = path[i];
            i += 1;
        }

        Ok(i)
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        match self.vec.upgrade() {
            Some(vec_intex) => {
                let mut vec = vec_intex.lock();
                let mut i = 0;
                while i < buf.len() {
                    vec.push_back(buf[i]);
                    i += 1;
                }
                Ok(i)
            },
            None => Err(Error::new(EPIPE))
        }
    }
}
