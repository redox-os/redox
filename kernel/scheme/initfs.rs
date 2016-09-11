use collections::BTreeMap;

use syscall::{Error, Result};
use super::Scheme;

struct Handle {
    data: &'static [u8],
    seek: usize
}

pub struct InitFsScheme {
    next_id: usize,
    files: BTreeMap<&'static [u8], &'static [u8]>,
    handles: BTreeMap<usize, Handle>
}

impl InitFsScheme {
    pub fn new() -> InitFsScheme {
        let mut files: BTreeMap<&'static [u8], &'static [u8]> = BTreeMap::new();

        files.insert(b"bin/init", include_bytes!("../../build/userspace/init"));
        files.insert(b"bin/pcid", include_bytes!("../../build/userspace/pcid"));
        files.insert(b"etc/init.rc", b"echo testing\n");

        InitFsScheme {
            next_id: 0,
            files: files,
            handles: BTreeMap::new()
        }
    }
}

impl Scheme for InitFsScheme {
    fn open(&mut self, path: &[u8], _flags: usize) -> Result<usize> {
        let data = self.files.get(path).ok_or(Error::NoEntry)?;
        let id = self.next_id;
        self.next_id += 1;
        self.handles.insert(id, Handle {
            data: data,
            seek: 0
        });
        Ok(id)
    }

    /// Read the file `number` into the `buffer`
    ///
    /// Returns the number of bytes read
    fn read(&mut self, file: usize, buffer: &mut [u8]) -> Result<usize> {
        let mut handle = self.handles.get_mut(&file).ok_or(Error::BadFile)?;

        let mut i = 0;
        while i < buffer.len() && handle.seek < handle.data.len() {
            buffer[i] = handle.data[handle.seek];
            i += 1;
            handle.seek += 1;
        }

        Ok(i)
    }

    /// Write the `buffer` to the `file`
    ///
    /// Returns the number of bytes written
    fn write(&mut self, _file: usize, _buffer: &[u8]) -> Result<usize> {
        Err(Error::NotPermitted)
    }

    /// Close the file `number`
    fn close(&mut self, file: usize) -> Result<()> {
        self.handles.remove(&file).ok_or(Error::BadFile).and(Ok(()))
    }
}
