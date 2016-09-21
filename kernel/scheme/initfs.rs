use collections::BTreeMap;
use core::cmp;
use core::sync::atomic::{AtomicUsize, Ordering};
use spin::RwLock;

use syscall::error::*;
use syscall::flag::{SEEK_SET, SEEK_CUR, SEEK_END};
use syscall::scheme::Scheme;

#[path="../../build/userspace/initfs.rs"]
mod gen;

struct Handle {
    data: &'static [u8],
    seek: usize
}

pub struct InitFsScheme {
    next_id: AtomicUsize,
    files: BTreeMap<&'static [u8], &'static [u8]>,
    handles: RwLock<BTreeMap<usize, Handle>>
}

impl InitFsScheme {
    pub fn new() -> InitFsScheme {
        InitFsScheme {
            next_id: AtomicUsize::new(0),
            files: gen::gen(),
            handles: RwLock::new(BTreeMap::new())
        }
    }
}

impl Scheme for InitFsScheme {
    fn open(&self, path: &[u8], _flags: usize) -> Result<usize> {
        let data = self.files.get(path).ok_or(Error::new(ENOENT))?;

        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        self.handles.write().insert(id, Handle {
            data: data,
            seek: 0
        });

        Ok(id)
    }

    fn dup(&self, id: usize) -> Result<usize> {
        let (data, seek) = {
            let handles = self.handles.read();
            let handle = handles.get(&id).ok_or(Error::new(EBADF))?;
            (handle.data, handle.seek)
        };

        let new_id = self.next_id.fetch_add(1, Ordering::SeqCst);
        self.handles.write().insert(new_id, Handle {
            data: data,
            seek: seek
        });

        Ok(new_id)
    }

    fn read(&self, id: usize, buf: &mut [u8]) -> Result<usize> {
        let mut handles = self.handles.write();
        let mut handle = handles.get_mut(&id).ok_or(Error::new(EBADF))?;

        let mut i = 0;
        while i < buf.len() && handle.seek < handle.data.len() {
            buf[i] = handle.data[handle.seek];
            i += 1;
            handle.seek += 1;
        }

        Ok(i)
    }

    fn seek(&self, id: usize, pos: usize, whence: usize) -> Result<usize> {
        let mut handles = self.handles.write();
        let mut handle = handles.get_mut(&id).ok_or(Error::new(EBADF))?;

        handle.seek = match whence {
            SEEK_SET => cmp::min(handle.data.len(), pos),
            SEEK_CUR => cmp::max(0, cmp::min(handle.data.len() as isize, handle.seek as isize + pos as isize)) as usize,
            SEEK_END => cmp::max(0, cmp::min(handle.data.len() as isize, handle.data.len() as isize + pos as isize)) as usize,
            _ => return Err(Error::new(EINVAL))
        };

        Ok(handle.seek)
    }

    fn fsync(&self, _id: usize) -> Result<usize> {
        Ok(0)
    }

    fn close(&self, id: usize) -> Result<usize> {
        self.handles.write().remove(&id).ok_or(Error::new(EBADF)).and(Ok(0))
    }
}
