use std::io::{Result, SeekFrom};
use std::syscall::*;
use std::url::Url;

pub struct Resource;

impl Resource {
    pub fn dup(&self) -> Result<Box<Resource>> {
        Ok(box Resource)
    }

    pub fn path(&self) -> Result<String> {
        Ok(format!(""))
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        Ok(0)
    }

    pub fn write(&mut self, buf: &[u8]) -> Result<usize> {
        Ok(0)
    }

    pub fn seek(&mut self, _: SeekFrom) -> Result<u64> {
        Err(SysError::new(ESPIPE))
    }

    pub fn sync(&mut self) -> Result<()> {
        Ok(())
    }
}

pub struct Scheme;

impl Scheme {
    pub fn new() -> Box<Scheme> {
        box Scheme
    }

    pub fn open(&mut self, url_str: &str, _: usize) -> Result<Box<Resource>> {
        Err(SysError::new(ENOENT))
    }
}
