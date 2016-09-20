use collections::BTreeMap;
use core::sync::atomic::{AtomicUsize, Ordering};
use spin::RwLock;

use syscall::error::*;
use syscall::scheme::Scheme;

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
        let mut files: BTreeMap<&'static [u8], &'static [u8]> = BTreeMap::new();

        files.insert(b"bin/init", include_bytes!("../../build/userspace/init"));
        files.insert(b"bin/ion", include_bytes!("../../build/userspace/ion"));
        files.insert(b"bin/pcid", include_bytes!("../../build/userspace/pcid"));
        files.insert(b"bin/ps2d", include_bytes!("../../build/userspace/ps2d"));
        files.insert(b"bin/example", include_bytes!("../../build/userspace/example"));
        files.insert(b"etc/init.rc", b"initfs:bin/pcid\ninitfs:bin/ps2d\ninitfs:bin/example\ninitfs:bin/ion");

        InitFsScheme {
            next_id: AtomicUsize::new(0),
            files: files,
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

    fn dup(&self, file: usize) -> Result<usize> {
        let (data, seek) = {
            let handles = self.handles.read();
            let handle = handles.get(&file).ok_or(Error::new(EBADF))?;
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
        let mut handle = handles.get_mut(&file).ok_or(Error::new(EBADF))?;

        let mut i = 0;
        while i < buffer.len() && handle.seek < handle.data.len() {
            buffer[i] = handle.data[handle.seek];
            i += 1;
            handle.seek += 1;
        }

        Ok(i)
    }

    fn fsync(&self, _file: usize) -> Result<usize> {
        Ok(0)
    }

    fn close(&self, file: usize) -> Result<usize> {
        self.handles.write().remove(&file).ok_or(Error::new(EBADF)).and(Ok(0))
    }
}
