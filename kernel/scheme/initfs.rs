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
        files.insert(b"bin/ion", include_bytes!("../../build/userspace/ion"));
        files.insert(b"bin/pcid", include_bytes!("../../build/userspace/pcid"));
        files.insert(b"bin/ps2d", include_bytes!("../../build/userspace/ps2d"));
        files.insert(b"etc/init.rc", b"#initfs:bin/pcid\ninitfs:bin/ps2d\n#initfs:bin/ion");

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

    fn dup(&mut self, file: usize) -> Result<usize> {
        let (data, seek) = {
            let handle = self.handles.get(&file).ok_or(Error::BadFile)?;
            (handle.data, handle.seek)
        };

        let id = self.next_id;
        self.next_id += 1;
        self.handles.insert(id, Handle {
            data: data,
            seek: seek
        });

        Ok(id)
    }

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

    fn write(&mut self, _file: usize, _buffer: &[u8]) -> Result<usize> {
        Err(Error::NotPermitted)
    }

    fn fsync(&mut self, _file: usize) -> Result<()> {
        Ok(())
    }

    fn close(&mut self, file: usize) -> Result<()> {
        self.handles.remove(&file).ok_or(Error::BadFile).and(Ok(()))
    }
}
