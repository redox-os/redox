use super::{Resource, ResourceSeek};

use alloc::boxed::Box;

use collections::{String, Vec};
use collections::string::ToString;

use core::cmp::{max, min};

use system::error::Result;

/// A vector resource
pub struct VecResource {
    path: String,
    data: Vec<u8>,
    seek: usize,
}

impl VecResource {
    pub fn new(path: &str, data: Vec<u8>) -> Self {
        VecResource {
            path: path.to_string(),
            data: data,
            seek: 0,
        }
    }

    pub fn data(&self) -> &Vec<u8> {
        return &self.data;
    }
}

impl Resource for VecResource {
    fn dup(&self) -> Result<Box<Resource>> {
        Ok(box VecResource {
            path: self.path.clone(),
            data: self.data.clone(),
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
        while i < buf.len() {
            self.data.push(buf[i]);
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

    fn truncate(&mut self, len: usize) -> Result<()> {
        while len > self.data.len() {
            self.data.push(0);
        }
        self.data.truncate(len);
        self.seek = min(self.seek, self.data.len());
        Ok(())
    }
}
