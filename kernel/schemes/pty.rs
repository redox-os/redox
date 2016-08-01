use alloc::arc::{Arc, Weak};
use alloc::boxed::Box;

use collections::{Vec, VecDeque};

use core::cmp;

use fs::{KScheme, Resource, Url};

use sync::WaitQueue;

use system::error::{Error, ENOENT, Result};

pub struct Pty {
    id: usize,
    input: WaitQueue<u8>,
    output: WaitQueue<Vec<u8>>
}

impl Pty {
    fn new(id: usize) -> Self {
        Pty {
            id: id,
            input: WaitQueue::new(),
            output: WaitQueue::new()
        }
    }
}

/// Psuedoterminal scheme
pub struct PtyScheme {
    next_id: usize,
    ptys: VecDeque<Weak<Pty>>
}

impl PtyScheme {
    pub fn new() -> Box<Self> {
        Box::new(PtyScheme {
            next_id: 1,
            ptys: VecDeque::new()
        })
    }
}

impl KScheme for PtyScheme {
    fn scheme(&self) -> &str {
        "pty"
    }

    fn open(&mut self, url: Url, _: usize) -> Result<Box<Resource>> {
        let req_id = url.reference().parse::<usize>().unwrap_or(0);

        self.ptys.retain(|pty| {
            pty.upgrade().is_some()
        });

        if req_id == 0 {
            let master = PtyMaster::new(self.next_id);

            self.ptys.push_back(Arc::downgrade(&master.inner));

            self.next_id += 1;
            //TODO: collision and rollover check

            Ok(Box::new(master))
        } else {
            for pty in self.ptys.iter() {
                if let Some(pty_strong) = pty.upgrade() {
                    if pty_strong.id == req_id {
                        return Ok(Box::new(PtySlave::new(&pty_strong)))
                    }
                }
            }

            Err(Error::new(ENOENT))
        }
    }
}

/// Psuedoterminal master
pub struct PtyMaster {
    inner: Arc<Pty>
}

impl PtyMaster {
    pub fn new(id: usize) -> Self {
        PtyMaster {
            inner: Arc::new(Pty::new(id))
        }
    }
}

impl Resource for PtyMaster {
    fn dup(&self) -> Result<Box<Resource>> {
        Ok(box PtyMaster {
            inner: self.inner.clone()
        })
    }

    fn path(&self, buf: &mut [u8]) -> Result<usize> {
        let path = format!("pty:{}", self.inner.id);

        for (b, p) in buf.iter_mut().zip(path.bytes()) {
            *b = p;
        }

        Ok(cmp::min(buf.len(), path.len()))
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let packet = self.inner.output.receive("PtyMaster::read");

        let mut i = 0;

        while i < buf.len() && i < packet.len() {
            buf[i] = packet[i];
            i += 1;
        }

        Ok(i)
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        for &b in buf.iter() {
            self.inner.input.send(b, "PtyMaster::write");
        }

        Ok(buf.len())
    }
}

/// Psuedoterminal slave
pub struct PtySlave {
    inner: Weak<Pty>
}

impl PtySlave {
    pub fn new(pty: &Arc<Pty>) -> Self {
        PtySlave {
            inner: Arc::downgrade(&pty)
        }
    }
}

impl Resource for PtySlave {
    fn dup(&self) -> Result<Box<Resource>> {
        Ok(box PtySlave {
            inner: self.inner.clone()
        })
    }

    fn path(&self, buf: &mut [u8]) -> Result<usize> {
        match self.inner.upgrade() {
            Some(inner) => {
                let path = format!("pty:{}", inner.id);

                for (b, p) in buf.iter_mut().zip(path.bytes()) {
                    *b = p;
                }

                Ok(cmp::min(buf.len(), path.len()))
            },
            None => Ok(0)
        }
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        match self.inner.upgrade() {
            Some(inner) => {
                if ! buf.is_empty() {
                    buf[0] = inner.input.receive("PtySlave::read");
                }

                let mut i = 1;

                while i < buf.len() {
                    match unsafe { inner.input.inner() }.pop_front() {
                        Some(b) => {
                            buf[i] = b;
                            i += 1;
                        },
                        None => break
                    }
                }

                Ok(i)
            },
            None => Ok(0)
        }
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        match self.inner.upgrade() {
            Some(inner) => {
                for chunk in buf.chunks(4095) {
                    let mut vec = vec![0];
                    vec.extend_from_slice(chunk);
                    inner.output.send(vec, "PtySlave::write");
                }

                Ok(buf.len())
            },
            None => Ok(0)
        }
    }

    fn sync(&mut self) -> Result<()> {
        if let Some(inner) = self.inner.upgrade() {
            inner.output.send(vec![1], "PtySlave::sync");
        }
        Ok(())
    }
}
