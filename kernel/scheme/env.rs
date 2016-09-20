use collections::BTreeMap;
use core::sync::atomic::{AtomicUsize, Ordering};
use spin::RwLock;

use syscall::{Error, Result};
use super::Scheme;

struct Handle {
    data: &'static [u8],
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

        files.insert(b"HOME", b"initfs:");
        files.insert(b"PWD", b"initfs:");
        files.insert(b"COLUMNS", b"80");
        files.insert(b"LINES", b"30");

        EnvScheme {
            next_id: AtomicUsize::new(0),
            files: files,
            handles: RwLock::new(BTreeMap::new())
        }
    }
}

impl Scheme for EnvScheme {
    fn open(&self, path: &[u8], _flags: usize) -> Result<usize> {
        let data = self.files.get(path).ok_or(Error::NoEntry)?;

        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        self.handles.write().insert(id, Handle {
            data: data,
            seek: 0
        });

        Ok(id)
    }

    fn dup(&self, file: usize) -> Result<usize> {
        let (data, seek) = {
            let handles = self.handles.read();
            let handle = handles.get(&file).ok_or(Error::BadFile)?;
            (handle.data, handle.seek)
        };

        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        self.handles.write().insert(id, Handle {
            data: data,
            seek: seek
        });

        Ok(id)
    }

    fn read(&self, file: usize, buffer: &mut [u8]) -> Result<usize> {
        let mut handles = self.handles.write();
        let mut handle = handles.get_mut(&file).ok_or(Error::BadFile)?;

        let mut i = 0;
        while i < buffer.len() && handle.seek < handle.data.len() {
            buffer[i] = handle.data[handle.seek];
            i += 1;
            handle.seek += 1;
        }

        Ok(i)
    }

    fn write(&self, _file: usize, _buffer: &[u8]) -> Result<usize> {
        Err(Error::NotPermitted)
    }

    fn fsync(&self, _file: usize) -> Result<()> {
        Ok(())
    }

    fn close(&self, file: usize) -> Result<()> {
        self.handles.write().remove(&file).ok_or(Error::BadFile).and(Ok(()))
    }
}
