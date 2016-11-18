use alloc::arc::Arc;
use collections::{BTreeMap, Vec};
use core::{cmp, str};
use core::sync::atomic::{AtomicUsize, Ordering};
use spin::RwLock;

use syscall::data::Stat;
use syscall::error::*;
use syscall::flag::{MODE_DIR, MODE_FILE, SEEK_SET, SEEK_CUR, SEEK_END};
use syscall::scheme::Scheme;

#[cfg(test)]
mod gen {
    use collections::BTreeMap;
    pub fn gen() -> BTreeMap<&'static [u8], (&'static [u8], bool)> { BTreeMap::new() }
}

#[cfg(not(test))]
#[path="../../build/userspace/initfs.rs"]
mod gen;

struct Handle {
    path: &'static [u8],
    data: Arc<RwLock<Vec<u8>>>,
    mode: u16,
    seek: usize
}

pub struct InitFsScheme {
    next_id: AtomicUsize,
    files: BTreeMap<&'static [u8], (Arc<RwLock<Vec<u8>>>, bool)>,
    handles: RwLock<BTreeMap<usize, Handle>>
}

impl InitFsScheme {
    pub fn new() -> InitFsScheme {
        let mut files = BTreeMap::new();
        for (path, (data, folder)) in gen::gen() {
            files.insert(path, (Arc::new(RwLock::new(data.to_vec())), folder));
        }

        InitFsScheme {
            next_id: AtomicUsize::new(0),
            files: files,
            handles: RwLock::new(BTreeMap::new())
        }
    }
}

impl Scheme for InitFsScheme {
    fn open(&self, path: &[u8], _flags: usize, _uid: u32, _gid: u32) -> Result<usize> {
        let path_utf8 = str::from_utf8(path).map_err(|_err| Error::new(ENOENT))?;
        let path_trimmed = path_utf8.trim_matches('/');

        //Have to iterate to get the path without allocation
        for entry in self.files.iter() {
            if entry.0 == &path_trimmed.as_bytes() {
                let id = self.next_id.fetch_add(1, Ordering::SeqCst);
                self.handles.write().insert(id, Handle {
                    path: entry.0,
                    data: (entry.1).0.clone(),
                    mode: if (entry.1).1 { MODE_DIR |  0o755 } else { MODE_FILE | 0o744 },
                    seek: 0
                });

                return Ok(id)
            }
        }

        Err(Error::new(ENOENT))
    }

    fn dup(&self, id: usize, _buf: &[u8]) -> Result<usize> {
        let (path, data, mode, seek) = {
            let handles = self.handles.read();
            let handle = handles.get(&id).ok_or(Error::new(EBADF))?;
            (handle.path, handle.data.clone(), handle.mode, handle.seek)
        };

        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        self.handles.write().insert(id, Handle {
            path: path,
            data: data,
            mode: mode,
            seek: seek
        });

        Ok(id)
    }

    fn read(&self, id: usize, buffer: &mut [u8]) -> Result<usize> {
        let mut handles = self.handles.write();
        let mut handle = handles.get_mut(&id).ok_or(Error::new(EBADF))?;
        let data = handle.data.read();

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
        let mut data = handle.data.write();

        let mut i = 0;
        while i < buffer.len() && handle.seek < data.len() {
            data[handle.seek] = buffer[i];
            i += 1;
            handle.seek += 1;
        }

        Ok(i)
    }

    fn seek(&self, id: usize, pos: usize, whence: usize) -> Result<usize> {
        let mut handles = self.handles.write();
        let mut handle = handles.get_mut(&id).ok_or(Error::new(EBADF))?;
        let data = handle.data.read();

        handle.seek = match whence {
            SEEK_SET => cmp::min(data.len(), pos),
            SEEK_CUR => cmp::max(0, cmp::min(data.len() as isize, handle.seek as isize + pos as isize)) as usize,
            SEEK_END => cmp::max(0, cmp::min(data.len() as isize, data.len() as isize + pos as isize)) as usize,
            _ => return Err(Error::new(EINVAL))
        };

        Ok(handle.seek)
    }

    fn fpath(&self, id: usize, buf: &mut [u8]) -> Result<usize> {
        let handles = self.handles.read();
        let handle = handles.get(&id).ok_or(Error::new(EBADF))?;

        //TODO: Copy scheme part in kernel
        let mut i = 0;
        let scheme_path = b"initfs:";
        while i < buf.len() && i < scheme_path.len() {
            buf[i] = scheme_path[i];
            i += 1;
        }

        let mut j = 0;
        while i < buf.len() && j < handle.path.len() {
            buf[i] = handle.path[j];
            i += 1;
            j += 1;
        }

        Ok(i)
    }

    fn fstat(&self, id: usize, stat: &mut Stat) -> Result<usize> {
        let handles = self.handles.read();
        let handle = handles.get(&id).ok_or(Error::new(EBADF))?;
        let data = handle.data.read();

        stat.st_mode = handle.mode;
        stat.st_uid = 0;
        stat.st_gid = 0;
        stat.st_size = data.len() as u64;

        Ok(0)
    }

    fn fsync(&self, id: usize) -> Result<usize> {
        let handles = self.handles.read();
        let _handle = handles.get(&id).ok_or(Error::new(EBADF))?;

        Ok(0)
    }

    fn close(&self, id: usize) -> Result<usize> {
        self.handles.write().remove(&id).ok_or(Error::new(EBADF)).and(Ok(0))
    }
}
