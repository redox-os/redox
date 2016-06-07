use alloc::arc::{Arc, Weak};
use alloc::boxed::Box;

use core::cmp;

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

    fn path(&self, buf: &mut [u8]) -> Result<usize> {
        let path = b"pipe:r";

        for (b, p) in buf.iter_mut().zip(path.iter()) {
            *b = *p;
        }

        Ok(cmp::min(buf.len(), path.len()))
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if Arc::weak_count(&self.vec) == 0 && unsafe { self.vec.inner() }.is_empty() {
            Ok(0)
        } else {
            if !buf.is_empty() {
                buf[0] = self.vec.receive();
            }

            let mut i = 1;

            while i < buf.len() {
                match unsafe { self.vec.inner() }.pop_front() {
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

    fn path(&self, buf: &mut [u8]) -> Result<usize> {
        let path = b"pipe:w";

        for (b, p) in buf.iter_mut().zip(path.iter()) {
            *b = *p;
        }

        Ok(cmp::min(buf.len(), path.len()))
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        match self.vec.upgrade() {
            Some(vec) => {
                for &b in buf.iter() {
                    vec.send(b);
                }

                Ok(buf.len())
            },
            None => Err(Error::new(EPIPE))
        }
    }

    fn sync(&mut self) -> Result<()> {
        //TODO: Wait until empty
        Ok(())
    }
}
