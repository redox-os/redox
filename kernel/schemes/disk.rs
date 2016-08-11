use alloc::arc::Arc;
use alloc::boxed::Box;

use collections::borrow::ToOwned;
use collections::{String, Vec};

use core::cell::UnsafeCell;
use core::cmp;
use disk::Disk;
use fs::{KScheme, Resource, ResourceSeek, VecResource};

use syscall::{MODE_DIR, MODE_FILE, Stat};

use system::error::{Error, Result, ENOENT};

/// A disk resource
pub struct DiskResource {
    pub path: String,
    pub disk: Arc<UnsafeCell<Box<Disk>>>,
    pub seek: u64,
}

impl Resource for DiskResource {
    fn dup(&self) -> Result<Box<Resource>> {
        Ok(box DiskResource {
            path: self.path.clone(),
            disk: self.disk.clone(),
            seek: self.seek,
        })
    }

    fn path(&self, buf: &mut [u8]) -> Result<usize> {
        let path = self.path.as_bytes();
        for (b, p) in buf.iter_mut().zip(path.iter()) {
            *b = *p;
        }

        Ok(cmp::min(buf.len(), path.len()))
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let count = try!(unsafe { &mut *self.disk.get() }.read(self.seek/512, buf));
        self.seek += count as u64;
        Ok(count)
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let count = try!(unsafe { &mut *self.disk.get() }.write(self.seek/512, buf));
        self.seek += count as u64;
        Ok(count)
    }

    fn seek(&mut self, pos: ResourceSeek) -> Result<usize> {
        let size = unsafe { & *self.disk.get() }.size();
        match pos {
            ResourceSeek::Start(offset) => self.seek = cmp::min(size, offset as u64),
            ResourceSeek::Current(offset) => self.seek = cmp::min(size, cmp::max(0, self.seek as i64 + offset as i64) as u64),
            ResourceSeek::End(offset) => self.seek = cmp::min(size, cmp::max(0, size as i64 + offset as i64) as u64),
        }
        Ok(self.seek as usize)
    }

    fn stat(&self, stat: &mut Stat) -> Result<()> {
        stat.st_size = unsafe { & *self.disk.get() }.size() as u32;
        stat.st_mode = MODE_FILE;
        Ok(())
    }

    fn sync(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Drop for DiskResource {
    fn drop(&mut self) {
        let _ = self.sync();
    }
}

/// A disk scheme
pub struct DiskScheme {
    disks: Vec<Arc<UnsafeCell<Box<Disk>>>>,
}

impl DiskScheme {
    /// Create a new disk scheme from an array of Disks
    pub fn new(mut disks: Vec<Box<Disk>>) -> Box<Self> {
        let mut scheme = box DiskScheme {
            disks: Vec::new()
        };

        for disk in disks.drain(..) {
            scheme.disks.push(Arc::new(UnsafeCell::new(disk)));
        }

        scheme
    }
}

impl KScheme for DiskScheme {
    fn scheme(&self) -> &str {
        "disk"
    }

    fn on_irq(&mut self, irq: u8) {
        for disk in self.disks.iter_mut() {
            unsafe { &mut *disk.get() }.on_irq(irq);
        }
    }

    fn open(&mut self, url: &str, _flags: usize) -> Result<Box<Resource>> {
        let path = url.splitn(2, ":").nth(1).unwrap_or("").trim_matches('/');

        if path.is_empty() {
            let mut list = String::new();
            for i in 0..self.disks.len() {
                if ! list.is_empty() {
                    list.push('\n');
                }
                list.push_str(&format!("{}", i));
            }

            return Ok(box VecResource::new("disk:/".to_owned(), list.into_bytes(), MODE_DIR));
        } else {
            if let Ok(number) = path.parse::<usize>() {
                if let Some(disk) = self.disks.get(number) {
                    return Ok(box DiskResource {
                        path: format!("disk:/{}", number),
                        disk: disk.clone(),
                        seek: 0
                    });
                }
            }
        }

        Err(Error::new(ENOENT))
    }
}
