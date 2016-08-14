//! Filesystem syscalls

use super::Result;

/// Read syscall
pub fn read(fd: usize, buf: &mut [u8]) -> Result<usize> {
    println!("Read {}: {}", fd, buf.len());
    Ok(0)
}

/// Write syscall
pub fn write(fd: usize, buf: &[u8]) -> Result<usize> {
    println!("Write {}: {}", fd, buf.len());
    Ok(0)
}

/// Open syscall
pub fn open(path: &[u8], flags: usize) -> Result<usize> {
    println!("Open {:?}: {:X}", ::core::str::from_utf8(path), flags);
    Ok(0)
}

/// Close syscall
pub fn close(fd: usize) -> Result<usize> {
    println!("Close {}", fd);
    Ok(0)
}
