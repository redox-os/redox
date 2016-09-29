use alloc::arc::Arc;
use collections::{BTreeMap, Vec};
use core::{cmp, str};
use core::sync::atomic::{AtomicUsize, Ordering};
use spin::{Mutex, RwLock};

use context;
use syscall::data::Stat;
use syscall::error::*;
use syscall::flag::{MODE_FILE, SEEK_SET, SEEK_CUR, SEEK_END};
use syscall::scheme::Scheme;

#[derive(Clone)]
struct Handle {
    data: Arc<Mutex<Vec<u8>>>,
    mode: u16,
    seek: usize
}

pub struct EnvScheme {
    next_id: AtomicUsize,
    handles: RwLock<BTreeMap<usize, Handle>>
}

impl EnvScheme {
    pub fn new() -> EnvScheme {
        EnvScheme {
            next_id: AtomicUsize::new(0),
            handles: RwLock::new(BTreeMap::new())
        }
    }
}

impl Scheme for EnvScheme {
    fn open(&self, path: &[u8], _flags: usize) -> Result<usize> {
        let path = str::from_utf8(path).map_err(|_err| Error::new(ENOENT))?.trim_matches('/');

        let env_lock = {
            let contexts = context::contexts();
            let context_lock = contexts.current().ok_or(Error::new(ESRCH))?;
            let context = context_lock.read();
            context.env.clone()
        };

        if path.is_empty() {
            let mut list = Vec::new();
            {
                let env = env_lock.lock();
                for entry in env.iter() {
                    if ! list.is_empty() {
                        list.push(b'\n');
                    }
                    list.extend_from_slice(&entry.0);
                    list.push(b'=');
                    list.extend_from_slice(&entry.1.lock());
                }
            }

            let id = self.next_id.fetch_add(1, Ordering::SeqCst);
            self.handles.write().insert(id, Handle {
                data: Arc::new(Mutex::new(list)),
                mode: MODE_FILE,
                seek: 0
            });

            Ok(id)
        } else {
            let data = {
                let mut env = env_lock.lock();
                if env.contains_key(path.as_bytes()) {
                    env[path.as_bytes()].clone()
                } else /*if flags & O_CREAT == O_CREAT*/ {
                    let name = path.as_bytes().to_vec().into_boxed_slice();
                    let data = Arc::new(Mutex::new(Vec::new()));
                    env.insert(name, data.clone());
                    data
                }
            };

            let id = self.next_id.fetch_add(1, Ordering::SeqCst);
            self.handles.write().insert(id, Handle {
                data: data,
                mode: MODE_FILE,
                seek: 0
            });

            Ok(id)
        }
    }

    fn dup(&self, id: usize) -> Result<usize> {
        let new_handle = {
            let handles = self.handles.read();
            let handle = handles.get(&id).ok_or(Error::new(EBADF))?;
            handle.clone()
        };

        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        self.handles.write().insert(id, new_handle);

        Ok(id)
    }

    fn read(&self, id: usize, buffer: &mut [u8]) -> Result<usize> {
        let mut handles = self.handles.write();
        let mut handle = handles.get_mut(&id).ok_or(Error::new(EBADF))?;

        let data = handle.data.lock();

        let mut i = 0;
        while i < buffer.len() && handle.seek < data.len() {
            buffer[i] = data[handle.seek];
            i += 1;
            handle.seek += 1;
        }

        Ok(i)
    }

    fn write(&self, id: usize, buffer: &[u8]) -> Result<usize> {
        let mut handles = self.handles.write();
        let mut handle = handles.get_mut(&id).ok_or(Error::new(EBADF))?;

        let mut data = handle.data.lock();

        let mut i = 0;
        while i < buffer.len() && handle.seek < data.len() {
            data[handle.seek] = buffer[i];
            i += 1;
            handle.seek += 1;
        }

        while i < buffer.len() {
            data.push(buffer[i]);
            i += 1;
            handle.seek += 1;
        }

        Ok(i)
    }

    fn seek(&self, id: usize, pos: usize, whence: usize) -> Result<usize> {
        let mut handles = self.handles.write();
        let mut handle = handles.get_mut(&id).ok_or(Error::new(EBADF))?;

        let len = handle.data.lock().len();
        handle.seek = match whence {
            SEEK_SET => cmp::min(len, pos),
            SEEK_CUR => cmp::max(0, cmp::min(len as isize, handle.seek as isize + pos as isize)) as usize,
            SEEK_END => cmp::max(0, cmp::min(len as isize, len as isize + pos as isize)) as usize,
            _ => return Err(Error::new(EINVAL))
        };

        Ok(handle.seek)
    }

    fn fstat(&self, id: usize, stat: &mut Stat) -> Result<usize> {
        let handles = self.handles.read();
        let handle = handles.get(&id).ok_or(Error::new(EBADF))?;

        stat.st_mode = handle.mode;
        stat.st_size = handle.data.lock().len() as u64;

        Ok(0)
    }

    fn fsync(&self, id: usize) -> Result<usize> {
        let handles = self.handles.read();
        let _handle = handles.get(&id).ok_or(Error::new(EBADF))?;

        Ok(0)
    }

    fn ftruncate(&self, id: usize, len: usize) -> Result<usize> {
        let handles = self.handles.read();
        let handle = handles.get(&id).ok_or(Error::new(EBADF))?;

        let mut data = handle.data.lock();
        if len < data.len() {
            data.truncate(len)
        } else {
            while len > data.len() {
                data.push(0);
            }
        }

        Ok(0)
    }

    fn close(&self, id: usize) -> Result<usize> {
        self.handles.write().remove(&id).ok_or(Error::new(EBADF)).and(Ok(0))
    }
}
