use collections::BTreeMap;
use core::cmp;
use core::sync::atomic::{AtomicUsize, Ordering};
use spin::RwLock;

use syscall::data::Stat;
use syscall::error::*;
use syscall::flag::{MODE_DIR, MODE_FILE, SEEK_SET, SEEK_CUR, SEEK_END};
use syscall::scheme::Scheme;

struct Handle {
    data: &'static [u8],
    mode: u16,
    seek: usize
}

pub struct EnvScheme {
    next_id: AtomicUsize,
    files: BTreeMap<&'static [u8], &'static [u8]>,
    handles: RwLock<BTreeMap<usize, Handle>>
}

impl EnvScheme {
    pub fn new() -> EnvScheme {
        let mut files: BTreeMap<&'static [u8], &'static [u8]> = BTreeMap::new();

        files.insert(b"", b"COLUMNS\nHOME\nLINES\nPATH\nPWD");
        files.insert(b"COLUMNS", b"80");
        files.insert(b"LINES", b"30");
        files.insert(b"HOME", b"initfs:bin/");
        files.insert(b"PATH", b"initfs:bin/");
        files.insert(b"PWD", b"initfs:bin/");

        EnvScheme {
            next_id: AtomicUsize::new(0),
            files: files,
            handles: RwLock::new(BTreeMap::new())
        }
    }
}

impl Scheme for EnvScheme {
    fn open(&self, path: &[u8], _flags: usize) -> Result<usize> {
        let data = self.files.get(path).ok_or(Error::new(ENOENT))?;

        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        self.handles.write().insert(id, Handle {
            data: data,
            mode: if path.is_empty() { MODE_DIR } else { MODE_FILE },
            seek: 0
        });

        Ok(id)
    }

    fn dup(&self, id: usize) -> Result<usize> {
        let (data, mode, seek) = {
            let handles = self.handles.read();
            let handle = handles.get(&id).ok_or(Error::new(EBADF))?;
            (handle.data, handle.mode, handle.seek)
        };

        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        self.handles.write().insert(id, Handle {
            data: data,
            mode: mode,
            seek: seek
        });

        Ok(id)
    }

    fn read(&self, id: usize, buffer: &mut [u8]) -> Result<usize> {
        let mut handles = self.handles.write();
        let mut handle = handles.get_mut(&id).ok_or(Error::new(EBADF))?;

        let mut i = 0;
        while i < buffer.len() && handle.seek < handle.data.len() {
            buffer[i] = handle.data[handle.seek];
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

    fn fstat(&self, id: usize, stat: &mut Stat) -> Result<usize> {
        let handles = self.handles.read();
        let handle = handles.get(&id).ok_or(Error::new(EBADF))?;

        stat.st_mode = handle.mode;
        stat.st_size = handle.data.len() as u32; //TODO: st_size 64-bit

        Ok(0)
    }

    fn fsync(&self, _id: usize) -> Result<usize> {
        Ok(0)
    }

    fn close(&self, id: usize) -> Result<usize> {
        self.handles.write().remove(&id).ok_or(Error::new(EBADF)).and(Ok(0))
    }
}
