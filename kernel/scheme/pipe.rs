use alloc::arc::{Arc, Weak};
use collections::{BTreeMap, VecDeque};
use core::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};
use spin::{Mutex, Once, RwLock, RwLockReadGuard, RwLockWriteGuard};

use context;
use syscall::error::{Error, Result, EBADF, EPIPE};
use syscall::scheme::Scheme;

/// Pipes list
pub static PIPE_SCHEME_ID: AtomicUsize = ATOMIC_USIZE_INIT;
static PIPE_NEXT_ID: AtomicUsize = ATOMIC_USIZE_INIT;
static PIPES: Once<RwLock<(BTreeMap<usize, PipeRead>, BTreeMap<usize, PipeWrite>)>> = Once::new();

/// Initialize pipes, called if needed
fn init_pipes() -> RwLock<(BTreeMap<usize, PipeRead>, BTreeMap<usize, PipeWrite>)> {
    RwLock::new((BTreeMap::new(), BTreeMap::new()))
}

/// Get the global pipes list, const
fn pipes() -> RwLockReadGuard<'static, (BTreeMap<usize, PipeRead>, BTreeMap<usize, PipeWrite>)> {
    PIPES.call_once(init_pipes).read()
}

/// Get the global schemes list, mutable
fn pipes_mut() -> RwLockWriteGuard<'static, (BTreeMap<usize, PipeRead>, BTreeMap<usize, PipeWrite>)> {
    PIPES.call_once(init_pipes).write()
}

pub fn pipe(_flags: usize) -> (usize, usize) {
    let mut pipes = pipes_mut();
    let read_id = PIPE_NEXT_ID.fetch_add(1, Ordering::SeqCst);
    let read = PipeRead::new();
    let write_id = PIPE_NEXT_ID.fetch_add(1, Ordering::SeqCst);
    let write = PipeWrite::new(&read);
    pipes.0.insert(read_id, read);
    pipes.1.insert(write_id, write);
    (read_id, write_id)
}

pub struct PipeScheme;

impl Scheme for PipeScheme {
    fn dup(&self, id: usize) -> Result<usize> {
        let mut pipes = pipes_mut();

        let read_option = pipes.0.get(&id).map(|pipe| pipe.clone());
        if let Some(pipe) = read_option {
            let pipe_id = PIPE_NEXT_ID.fetch_add(1, Ordering::SeqCst);
            pipes.0.insert(pipe_id, pipe);
            return Ok(pipe_id);
        }

        let write_option = pipes.1.get(&id).map(|pipe| pipe.clone());
        if let Some(pipe) = write_option {
            let pipe_id = PIPE_NEXT_ID.fetch_add(1, Ordering::SeqCst);
            pipes.1.insert(pipe_id, pipe);
            return Ok(pipe_id);
        }

        Err(Error::new(EBADF))
    }

    fn read(&self, id: usize, buf: &mut [u8]) -> Result<usize> {
        let pipe_option = {
            let pipes = pipes();
            pipes.0.get(&id).map(|pipe| pipe.clone())
        };

        if let Some(pipe) = pipe_option {
            pipe.read(buf)
        } else {
            Err(Error::new(EBADF))
        }
    }

    fn write(&self, id: usize, buf: &[u8]) -> Result<usize> {
        let pipe_option = {
            let pipes = pipes();
            pipes.1.get(&id).map(|pipe| pipe.clone())
        };

        if let Some(pipe) = pipe_option {
            pipe.write(buf)
        } else {
            Err(Error::new(EBADF))
        }
    }

    fn fsync(&self, _id: usize) -> Result<usize> {
        Ok(0)
    }

    fn close(&self, id: usize) -> Result<usize> {
        let mut pipes = pipes_mut();

        drop(pipes.0.remove(&id));
        drop(pipes.1.remove(&id));

        Ok(0)
    }
}

/// Read side of a pipe
#[derive(Clone)]
pub struct PipeRead {
    vec: Arc<Mutex<VecDeque<u8>>>
}

impl PipeRead {
    pub fn new() -> Self {
        PipeRead {
            vec: Arc::new(Mutex::new(VecDeque::new()))
        }
    }

    fn read(&self, buf: &mut [u8]) -> Result<usize> {
        if buf.is_empty() || (Arc::weak_count(&self.vec) == 0 && self.vec.lock().is_empty()) {
            Ok(0)
        } else {
            /*loop {
                {
                    if let Some(byte) = self.vec.lock().pop_front() {
                        buf[0] = byte;
                        break;
                    }
                }
                unsafe { context::switch(); }
            }*/

            let mut i = 0;

            while i < buf.len() {
                match self.vec.lock().pop_front() {
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
#[derive(Clone)]
pub struct PipeWrite {
    vec: Weak<Mutex<VecDeque<u8>>>,
}

impl PipeWrite {
    pub fn new(read: &PipeRead) -> Self {
        PipeWrite {
            vec: Arc::downgrade(&read.vec),
        }
    }

    fn write(&self, buf: &[u8]) -> Result<usize> {
        match self.vec.upgrade() {
            Some(vec) => {
                for &b in buf.iter() {
                    vec.lock().push_back(b);
                }

                Ok(buf.len())
            },
            None => Err(Error::new(EPIPE))
        }
    }
}
