use super::{Resource, ResourceSeek};

use alloc::boxed::Box;

use core::cmp::{max, min};
use core::slice;

use system::error::Result;

/// A slice resource
pub struct SliceResource {
    path: &'static str,
    data: &'static [u8],
    seek: usize,
}

impl SliceResource {
    pub fn new(path: &'static str, data: &'static [u8]) -> Self {
        SliceResource {
            path: path,
            data: data,
            seek: 0,
        }
    }
}

impl Resource for SliceResource {
    fn dup(&self) -> Result<Box<Resource>> {
        Ok(box SliceResource {
            path: self.path,
            data: self.data,
            seek: self.seek,
        })
    }

    fn path(&self, buf: &mut [u8]) -> Result <usize> {
        let path = self.path.as_bytes();

        let mut i = 0;
        while i < buf.len() && i < path.len() {
            buf[i] = path[i];
            i += 1;
        }

        Ok(i)
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut i = 0;
        while i < buf.len() && self.seek < self.data.len() {
            match self.data.get(self.seek) {
                Some(b) => buf[i] = *b,
                None => (),
            }
            self.seek += 1;
            i += 1;
        }
        return Ok(i);
    }

    fn seek(&mut self, pos: ResourceSeek) -> Result<usize> {
        match pos {
            ResourceSeek::Start(offset) => self.seek = min(self.data.len(), offset),
            ResourceSeek::Current(offset) =>
                self.seek = max(0, min(self.seek as isize, self.seek as isize + offset)) as usize,
            ResourceSeek::End(offset) =>
                self.seek = max(0,
                                min(self.seek as isize,
                                    self.data.len() as isize +
                                    offset)) as usize,
        }
        return Ok(self.seek);
    }

    fn sync(&mut self) -> Result<()> {
        Ok(())
    }
}

/// A slice resource
pub struct SliceMutResource {
    path: &'static str,
    data: &'static mut [u8],
    seek: usize,
}

impl SliceMutResource {
    pub fn new(path: &'static str, data: &'static mut [u8]) -> Self {
        SliceMutResource {
            path: path,
            data: data,
            seek: 0,
        }
    }
}

impl Resource for SliceMutResource {
    fn dup(&self) -> Result<Box<Resource>> {
        Ok(box SliceMutResource {
            path: self.path,
            data: unsafe { slice::from_raw_parts_mut(self.data.as_ptr() as *mut u8, self.data.len()) },
            seek: self.seek,
        })
    }

    fn path(&self, buf: &mut [u8]) -> Result <usize> {
        let path = self.path.as_bytes();

        let mut i = 0;
        while i < buf.len() && i < path.len() {
            buf[i] = path[i];
            i += 1;
        }

        Ok(i)
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut i = 0;
        while i < buf.len() && self.seek < self.data.len() {
            match self.data.get(self.seek) {
                Some(b) => buf[i] = *b,
                None => (),
            }
            self.seek += 1;
            i += 1;
        }
        return Ok(i);
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let mut i = 0;
        while i < buf.len() && self.seek < self.data.len() {
            self.data[self.seek] = buf[i];
            self.seek += 1;
            i += 1;
        }
        return Ok(i);
    }

    fn seek(&mut self, pos: ResourceSeek) -> Result<usize> {
        match pos {
            ResourceSeek::Start(offset) => self.seek = min(self.data.len(), offset),
            ResourceSeek::Current(offset) =>
                self.seek = max(0, min(self.seek as isize, self.seek as isize + offset)) as usize,
            ResourceSeek::End(offset) =>
                self.seek = max(0,
                                min(self.seek as isize,
                                    self.data.len() as isize +
                                    offset)) as usize,
        }
        return Ok(self.seek);
    }

    fn sync(&mut self) -> Result<()> {
        Ok(())
    }
}
