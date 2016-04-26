use alloc::arc::Arc;
use alloc::boxed::Box;

use collections::borrow::ToOwned;
use collections::{String, Vec};

use core::cmp;
use disk::Disk;
use fs::{KScheme, Resource, ResourceSeek, Url, VecResource};
use sync::Intex;

use syscall::{MODE_DIR, MODE_FILE, Stat};

use system::error::{Error, Result, ENOENT};

/// A disk resource
pub struct DiskResource {
    pub path: String,
    pub disk: Arc<Intex<Box<Disk>>>,
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
        let count = try!(self.disk.lock().read(self.seek/512, buf));
        self.seek += count as u64;
        Ok(count)
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let count = try!(self.disk.lock().write(self.seek/512, buf));
        self.seek += count as u64;
        Ok(count)
    }

    fn seek(&mut self, pos: ResourceSeek) -> Result<usize> {
        let size = self.disk.lock().size();
        match pos {
            ResourceSeek::Start(offset) => self.seek = cmp::min(size, offset as u64),
            ResourceSeek::Current(offset) => self.seek = cmp::min(size, cmp::max(0, self.seek as i64 + offset as i64) as u64),
            ResourceSeek::End(offset) => self.seek = cmp::min(size, cmp::max(0, size as i64 + offset as i64) as u64),
        }
        Ok(self.seek as usize)
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
    disks: Vec<Arc<Intex<Box<Disk>>>>,
}

impl DiskScheme {
    /// Create a new disk scheme from an array of Disks
    pub fn new(mut disks: Vec<Box<Disk>>) -> Box<Self> {
        let mut scheme = box DiskScheme {
            disks: Vec::new()
        };

        for disk in disks.drain(..) {
            scheme.disks.push(Arc::new(Intex::new(disk)));
        }

        scheme
    }
}

impl KScheme for DiskScheme {
    fn on_irq(&mut self, _irq: u8) {
        //TODO
    }

    fn scheme(&self) -> &str {
        "disk"
    }

    fn open(&mut self, url: Url, _flags: usize) -> Result<Box<Resource>> {
        let path = url.reference().trim_matches('/');

        if path.is_empty() {
            let mut list = String::new();
            for i in 0..self.disks.len() {
                if ! list.is_empty() {
                    list.push('\n');
                }
                list.push_str(&format!("{}", i));
            }

            return Ok(box VecResource::new("disk:/".to_owned(), list.into_bytes()));
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

    fn stat(&mut self, url: Url, stat: &mut Stat) -> Result<()> {
        let path = url.reference().trim_matches('/');

        if path.is_empty() {
            let mut list = String::new();
            for i in 0..self.disks.len() {
                if ! list.is_empty() {
                    list.push('\n');
                }
                list.push_str(&format!("{}", i));
            }

            stat.st_mode = MODE_DIR;
            stat.st_size = list.len() as u64;
            return Ok(());
        } else {
            if let Ok(number) = path.parse::<usize>() {
                if let Some(disk) = self.disks.get(number) {
                    stat.st_mode = MODE_FILE;
                    stat.st_size = disk.lock().size();
                    return Ok(());
                }
            }
        }

        Err(Error::new(ENOENT))
    }
}
