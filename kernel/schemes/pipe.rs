use alloc::arc::{Arc, Weak};
use alloc::boxed::Box;

use collections::vec_deque::VecDeque;

use schemes::{Result, Resource, Url};

use scheduler::context::context_switch;

use sync::Intex;

use syscall::{SysError, EPIPE};

/// Read side of a pipe
pub struct PipeRead {
    vec: Arc<Intex<VecDeque<u8>>>,
}

impl PipeRead {
    pub fn new() -> Self {
        PipeRead {
            vec: Arc::new(Intex::new(VecDeque::new())),
        }
    }
}

impl Resource for PipeRead {
    fn dup(&self) -> Result<Box<Resource>> {
        Ok(box PipeRead {
            vec: self.vec.clone(),
        })
    }

    fn url(&self) -> Url {
        return Url::from_str("pipe:r");
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if Arc::weak_count(&self.vec) > 1 {
            while self.vec.lock().is_empty() {
                unsafe { context_switch(false) };
            }

            let mut i = 0;
            let mut vec = self.vec.lock();
            while i < buf.len() {
                match vec.pop_front() {
                    Some(b) => {
                        buf[i] = b;
                        i += 1;
                    },
                    None => break
                }
            }
            Ok(i)
        } else {
            Ok(0)
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

    fn url(&self) -> Url {
        return Url::from_str("pipe:w");
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        match self.vec.upgrade() {
            Some(vec_intex) => {
                let mut i = 0;
                let mut vec = vec_intex.lock();
                while i < buf.len() {
                    vec.push_back(buf[i]);
                }
                Ok(i)
            },
            None => Err(SysError::new(EPIPE))
        }
    }
}
