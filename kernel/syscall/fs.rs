//! Filesystem syscalls

use super::{Error, Result};

/// Read syscall
pub fn read(fd: usize, buf: &mut [u8]) -> Result<usize> {
    println!("Read {}: {}", fd, buf.len());
    if let Some(file) = unsafe { &mut ::context::CONTEXT }.files.get(fd) {
        println!("{:?}", file);
        Ok(0)
    } else {
        Err(Error::BadFile)
    }
}

/// Write syscall
pub fn write(fd: usize, buf: &[u8]) -> Result<usize> {
    println!("Write {}: {}", fd, buf.len());
    if let Some(file) = unsafe { &mut ::context::CONTEXT }.files.get(fd) {
        println!("{:?}: {:?}", file, ::core::str::from_utf8(buf));
        Ok(buf.len())
    } else {
        Err(Error::BadFile)
    }
}

/// Open syscall
pub fn open(path: &[u8], flags: usize) -> Result<usize> {
    println!("Open {:?}: {:X}", ::core::str::from_utf8(path), flags);
    if let Some(fd) = unsafe { &mut ::context::CONTEXT }.add_file(::context::file::File {
        scheme: 0,
        number: 0
    }) {
        Ok(fd)
    } else {
        Err(Error::TooManyFiles)
    }
}

/// Close syscall
pub fn close(fd: usize) -> Result<usize> {
    println!("Close {}", fd);
    Ok(0)
}
