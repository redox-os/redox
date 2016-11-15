use alloc::arc::{Arc, Weak};
use collections::{BTreeMap, VecDeque};
use core::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};
use spin::{Mutex, Once, RwLock, RwLockReadGuard, RwLockWriteGuard};

use sync::WaitCondition;
use syscall::error::{Error, Result, EAGAIN, EBADF, EINVAL, EPIPE};
use syscall::flag::{F_GETFL, F_SETFL, O_CLOEXEC, O_NONBLOCK};
use syscall::scheme::Scheme;

/// Pipes list
pub static PIPE_SCHEME_ID: AtomicUsize = ATOMIC_USIZE_INIT;
static PIPE_NEXT_ID: AtomicUsize = ATOMIC_USIZE_INIT;
static PIPES: Once<RwLock<(BTreeMap<usize, Arc<PipeRead>>, BTreeMap<usize, Arc<PipeWrite>>)>> = Once::new();

/// Initialize pipes, called if needed
fn init_pipes() -> RwLock<(BTreeMap<usize, Arc<PipeRead>>, BTreeMap<usize, Arc<PipeWrite>>)> {
    RwLock::new((BTreeMap::new(), BTreeMap::new()))
}

/// Get the global pipes list, const
fn pipes() -> RwLockReadGuard<'static, (BTreeMap<usize, Arc<PipeRead>>, BTreeMap<usize, Arc<PipeWrite>>)> {
    PIPES.call_once(init_pipes).read()
}

/// Get the global schemes list, mutable
fn pipes_mut() -> RwLockWriteGuard<'static, (BTreeMap<usize, Arc<PipeRead>>, BTreeMap<usize, Arc<PipeWrite>>)> {
    PIPES.call_once(init_pipes).write()
}

pub fn pipe(flags: usize) -> (usize, usize) {
    let mut pipes = pipes_mut();
    let read_id = PIPE_NEXT_ID.fetch_add(1, Ordering::SeqCst);
    let write_id = PIPE_NEXT_ID.fetch_add(1, Ordering::SeqCst);
    let read = PipeRead::new(flags);
    let write = PipeWrite::new(flags, &read);
    pipes.0.insert(read_id, Arc::new(read));
    pipes.1.insert(write_id, Arc::new(write));
    (read_id, write_id)
}

pub struct PipeScheme;

impl Scheme for PipeScheme {
    fn dup(&self, id: usize, buf: &[u8]) -> Result<usize> {
        let mut pipes = pipes_mut();

        let read_option = if let Some(pipe) = pipes.0.get(&id) {
            Some(pipe.dup(buf)?)
        } else {
            None
        };
        if let Some(pipe) = read_option {
            let pipe_id = PIPE_NEXT_ID.fetch_add(1, Ordering::SeqCst);
            pipes.0.insert(pipe_id, Arc::new(pipe));
            return Ok(pipe_id);
        }

        let write_option = if let Some(pipe) = pipes.1.get(&id) {
            Some(pipe.dup(buf)?)
        } else {
            None
        };
        if let Some(pipe) = write_option {
            let pipe_id = PIPE_NEXT_ID.fetch_add(1, Ordering::SeqCst);
            pipes.1.insert(pipe_id, Arc::new(pipe));
            return Ok(pipe_id);
        }

        Err(Error::new(EBADF))
    }

    fn read(&self, id: usize, buf: &mut [u8]) -> Result<usize> {
        // Clone to prevent deadlocks
        let pipe = {
            let pipes = pipes();
            pipes.0.get(&id).map(|pipe| pipe.clone()).ok_or(Error::new(EBADF))?
        };

        pipe.read(buf)
    }

    fn write(&self, id: usize, buf: &[u8]) -> Result<usize> {
        // Clone to prevent deadlocks
        let pipe = {
            let pipes = pipes();
            pipes.1.get(&id).map(|pipe| pipe.clone()).ok_or(Error::new(EBADF))?
        };

        pipe.write(buf)
    }

    fn fcntl(&self, id: usize, cmd: usize, arg: usize) -> Result<usize> {
        let pipes = pipes();

        if let Some(pipe) = pipes.0.get(&id) {
            return pipe.fcntl(cmd, arg);
        }

        if let Some(pipe) = pipes.1.get(&id) {
            return pipe.fcntl(cmd, arg);
        }

        Err(Error::new(EBADF))
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
pub struct PipeRead {
    flags: AtomicUsize,
    condition: Arc<WaitCondition>,
    vec: Arc<Mutex<VecDeque<u8>>>
}

impl PipeRead {
    pub fn new(flags: usize) -> Self {
        PipeRead {
            flags: AtomicUsize::new(flags),
            condition: Arc::new(WaitCondition::new()),
            vec: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    fn dup(&self, buf: &[u8]) -> Result<Self> {
        if buf == b"exec" && self.flags.load(Ordering::SeqCst) & O_CLOEXEC == O_CLOEXEC {
            Err(Error::new(EBADF))
        } else {
            let mut flags = self.flags.load(Ordering::SeqCst);
            if buf.is_empty() {
                flags &= ! O_CLOEXEC;
            }
            Ok(PipeRead {
                flags: AtomicUsize::new(flags),
                condition: self.condition.clone(),
                vec: self.vec.clone()
            })
        }
    }

    fn fcntl(&self, cmd: usize, arg: usize) -> Result<usize> {
        match cmd {
            F_GETFL => Ok(self.flags.load(Ordering::SeqCst)),
            F_SETFL => {
                self.flags.store(arg, Ordering::SeqCst);
                Ok(0)
            },
            _ => Err(Error::new(EINVAL))
        }
    }

    fn read(&self, buf: &mut [u8]) -> Result<usize> {
        loop {
            {
                let mut vec = self.vec.lock();

                let mut i = 0;
                while i < buf.len() {
                    if let Some(b) = vec.pop_front() {
                        buf[i] = b;
                        i += 1;
                    } else {
                        break;
                    }
                }

                if i > 0 {
                    return Ok(i);
                }
            }

            if Arc::weak_count(&self.vec) == 0 {
                return Ok(0);
            } else if self.flags.load(Ordering::SeqCst) & O_NONBLOCK == O_NONBLOCK {
                return Err(Error::new(EAGAIN));
            } else {
                self.condition.wait();
            }
        }
    }
}

/// Read side of a pipe
pub struct PipeWrite {
    flags: AtomicUsize,
    condition: Arc<WaitCondition>,
    vec: Option<Weak<Mutex<VecDeque<u8>>>>
}

impl PipeWrite {
    pub fn new(flags: usize, read: &PipeRead) -> Self {
        PipeWrite {
            flags: AtomicUsize::new(flags),
            condition: read.condition.clone(),
            vec: Some(Arc::downgrade(&read.vec)),
        }
    }

    fn dup(&self, buf: &[u8]) -> Result<Self> {
        if buf == b"exec" && self.flags.load(Ordering::SeqCst) & O_CLOEXEC == O_CLOEXEC {
            Err(Error::new(EBADF))
        } else {
            let mut flags = self.flags.load(Ordering::SeqCst);
            if buf.is_empty() {
                flags &= ! O_CLOEXEC;
            }
            Ok(PipeWrite {
                flags: AtomicUsize::new(flags),
                condition: self.condition.clone(),
                vec: self.vec.clone()
            })
        }
    }

    fn fcntl(&self, cmd: usize, arg: usize) -> Result<usize> {
        match cmd {
            F_GETFL => Ok(self.flags.load(Ordering::SeqCst)),
            F_SETFL => {
                self.flags.store(arg, Ordering::SeqCst);
                Ok(0)
            },
            _ => Err(Error::new(EINVAL))
        }
    }

    fn write(&self, buf: &[u8]) -> Result<usize> {
        if let Some(ref vec_weak) = self.vec {
            if let Some(vec_lock) = vec_weak.upgrade() {
                let mut vec = vec_lock.lock();

                for &b in buf.iter() {
                    vec.push_back(b);
                }

                self.condition.notify();

                Ok(buf.len())
            } else {
                Err(Error::new(EPIPE))
            }
        } else {
            panic!("PipeWrite dropped before write");
        }
    }
}

impl Drop for PipeWrite {
    fn drop(&mut self) {
        drop(self.vec.take());
        self.condition.notify();
    }
}
