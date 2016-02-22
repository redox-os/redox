use alloc::arc::{Arc, Weak};
use alloc::boxed::Box;

use fs::Resource;

use sync::WaitQueue;

use system::error::{Error, Result, EPIPE};

/// Read side of a pipe
pub struct PipeRead {
    vec: Arc<WaitQueue<u8>>
}

impl PipeRead {
    pub fn new() -> Self {
        PipeRead {
            vec: Arc::new(WaitQueue::new())
        }
    }
}

impl Resource for PipeRead {
    fn dup(&self) -> Result<Box<Resource>> {
        Ok(box PipeRead {
            vec: self.vec.clone(),
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
        if Arc::weak_count(&self.vec) == 0 && self.vec.inner.lock().is_empty() {
            Ok(0)
        } else {
            let mut i = 0;

            if i < buf.len() {
                buf[i] = self.vec.receive();
                i += 1;
            }

            while i < buf.len() {
                match self.vec.inner.lock().pop_front() {
                    Some(b) => {
                        buf[i] = b;
                        i += 1;
                    },
                    None => break
                }
            }

            Ok(i)
        }
    }
}

/// Read side of a pipe
pub struct PipeWrite {
    vec: Weak<WaitQueue<u8>>,
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
            Some(vec) => {
                let mut i = 0;

                while i < buf.len() {
                    vec.send(buf[i]);
                    i += 1;
                }

                Ok(i)
            },
            None => Err(Error::new(EPIPE))
        }
    }
}
