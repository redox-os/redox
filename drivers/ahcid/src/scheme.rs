use std::collections::BTreeMap;
use std::{cmp, str};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use spin::Mutex;
use syscall::{Error, EBADF, EINVAL, ENOENT, Result, Scheme, SEEK_CUR, SEEK_END, SEEK_SET};

use ahci::disk::Disk;

pub struct DiskScheme {
    disks: Box<[Arc<Mutex<Disk>>]>,
    handles: Mutex<BTreeMap<usize, (Arc<Mutex<Disk>>, usize)>>,
    next_id: AtomicUsize
}

impl DiskScheme {
    pub fn new(disks: Vec<Disk>) -> DiskScheme {
        let mut disk_arcs = vec![];
        for disk in disks {
            disk_arcs.push(Arc::new(Mutex::new(disk)));
        }

        DiskScheme {
            disks: disk_arcs.into_boxed_slice(),
            handles: Mutex::new(BTreeMap::new()),
            next_id: AtomicUsize::new(0)
        }
    }
}

impl Scheme for DiskScheme {
    fn open(&self, path: &[u8], _flags: usize) -> Result<usize> {
        let path_str = str::from_utf8(path).or(Err(Error::new(ENOENT)))?;

        let i = path_str.parse::<usize>().or(Err(Error::new(ENOENT)))?;

        if let Some(disk) = self.disks.get(i) {
            let id = self.next_id.fetch_add(1, Ordering::SeqCst);
            self.handles.lock().insert(id, (disk.clone(), 0));
            Ok(id)
        } else {
            Err(Error::new(ENOENT))
        }
    }

    fn read(&self, id: usize, buf: &mut [u8]) -> Result<usize> {
        let mut handles = self.handles.lock();
        let mut handle = handles.get_mut(&id).ok_or(Error::new(EBADF))?;

        let mut disk = handle.0.lock();
        let count = disk.read((handle.1 as u64)/512, buf)?;
        handle.1 += count;
        Ok(count)
    }

    fn write(&self, id: usize, buf: &[u8]) -> Result<usize> {
        let mut handles = self.handles.lock();
        let mut handle = handles.get_mut(&id).ok_or(Error::new(EBADF))?;

        let mut disk = handle.0.lock();
        let count = disk.write((handle.1 as u64)/512, buf)?;
        handle.1 += count;
        Ok(count)
    }

    fn seek(&self, id: usize, pos: usize, whence: usize) -> Result<usize> {
        let mut handles = self.handles.lock();
        let mut handle = handles.get_mut(&id).ok_or(Error::new(EBADF))?;

        let len = handle.0.lock().size() as usize;
        handle.1 = match whence {
            SEEK_SET => cmp::min(len, pos),
            SEEK_CUR => cmp::max(0, cmp::min(len as isize, handle.1 as isize + pos as isize)) as usize,
            SEEK_END => cmp::max(0, cmp::min(len as isize, len as isize + pos as isize)) as usize,
            _ => return Err(Error::new(EINVAL))
        };

        Ok(handle.1)
    }

    fn close(&self, id: usize) -> Result<usize> {
        let mut handles = self.handles.lock();
        handles.remove(&id).ok_or(Error::new(EBADF)).and(Ok(0))
    }
}
