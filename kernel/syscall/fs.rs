//! Filesystem syscalls

use context;
use scheme;

use super::{Error, Result};

/// Read syscall
pub fn read(fd: usize, buf: &mut [u8]) -> Result<usize> {
    println!("Read {}: {}", fd, buf.len());
    if let Some(context_lock) = context::contexts().current() {
        let context = context_lock.read();
        if let Some(file) = context.files.get(fd) {
            println!("{:?}", file);
            Ok(0)
        } else {
            Err(Error::BadFile)
        }
    } else {
        Err(Error::NoProcess)
    }
}

/// Write syscall
pub fn write(fd: usize, buf: &[u8]) -> Result<usize> {
    println!("Write {}: {}", fd, buf.len());
    if let Some(context_lock) = context::contexts().current() {
        let context = context_lock.read();
        if let Some(file) = context.files.get(fd) {
            println!("{:?}: {:?}", file, ::core::str::from_utf8(buf));
            Ok(buf.len())
        } else {
            Err(Error::BadFile)
        }
    } else {
        Err(Error::NoProcess)
    }
}

/// Open syscall
pub fn open(path: &[u8], flags: usize) -> Result<usize> {
    let mut parts = path.splitn(2, |&b| b == b':');
    let namespace_opt = parts.next();
    let reference_opt = parts.next();
    println!("Open namespace {:?} reference {:?}: {:X}", namespace_opt.map(::core::str::from_utf8), reference_opt.map(::core::str::from_utf8), flags);

    let file = {
        if let Some(namespace) = namespace_opt {
            let schemes = scheme::schemes();
            if let Some(scheme_mutex) = schemes.get(namespace) {
                scheme_mutex.lock().open(reference_opt.unwrap_or(b""), flags)
            } else {
                Err(Error::NoEntry)
            }
        } else {
            Err(Error::NoEntry)
        }
    }?;

    if let Some(context_lock) = context::contexts().current() {
        let mut context = context_lock.write();
        if let Some(fd) = context.add_file(::context::file::File {
            scheme: 0,
            number: file
        }) {
            Ok(fd)
        } else {
            Err(Error::TooManyFiles)
        }
    } else {
        Err(Error::NoProcess)
    }
}

/// Close syscall
pub fn close(fd: usize) -> Result<usize> {
    println!("Close {}", fd);
    Ok(0)
}
