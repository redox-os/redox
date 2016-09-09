//! Filesystem syscalls

use context;
use scheme;

use super::{Error, Result};

/// Read syscall
pub fn read(fd: usize, buf: &mut [u8]) -> Result<usize> {
    println!("Read {}: {:X} {}", fd, buf.as_ptr() as usize, buf.len());

    let file = {
        let contexts = context::contexts();
        let context_lock = contexts.current().ok_or(Error::NoProcess)?;
        let context = context_lock.read();
        let file = context.files.get(fd).ok_or(Error::BadFile)?.ok_or(Error::BadFile)?;
        file
    };

    println!("{:?}", file);

    let schemes = scheme::schemes();
    let scheme_mutex = schemes.get(file.scheme).ok_or(Error::BadFile)?;
    let result = scheme_mutex.lock().read(file.number, buf);
    result
}

/// Write syscall
pub fn write(fd: usize, buf: &[u8]) -> Result<usize> {
    println!("Write {}: {:X} {}", fd, buf.as_ptr() as usize, buf.len());

    let file = {
        let contexts = context::contexts();
        let context_lock = contexts.current().ok_or(Error::NoProcess)?;
        let context = context_lock.read();
        let file = context.files.get(fd).ok_or(Error::BadFile)?.ok_or(Error::BadFile)?;
        file
    };

    println!("{:?}: {:?}", file, ::core::str::from_utf8(buf));

    let schemes = scheme::schemes();
    let scheme_mutex = schemes.get(file.scheme).ok_or(Error::BadFile)?;
    let result = scheme_mutex.lock().write(file.number, buf);
    result
}

/// Open syscall
pub fn open(path: &[u8], flags: usize) -> Result<usize> {
    let mut parts = path.splitn(2, |&b| b == b':');
    let namespace_opt = parts.next();
    let reference_opt = parts.next();
    println!("Open namespace {:?} reference {:?}: {:X}", namespace_opt.map(::core::str::from_utf8), reference_opt.map(::core::str::from_utf8), flags);

    let (scheme_id, file_id) = {
        let namespace = namespace_opt.ok_or(Error::NoEntry)?;
        let schemes = scheme::schemes();
        let (scheme_id, scheme_mutex) = schemes.get_name(namespace).ok_or(Error::NoEntry)?;
        let file_id = scheme_mutex.lock().open(reference_opt.unwrap_or(b""), flags)?;
        (scheme_id, file_id)
    };

    let contexts = context::contexts();
    let context_lock = contexts.current().ok_or(Error::NoProcess)?;
    let mut context = context_lock.write();
    context.add_file(::context::file::File {
        scheme: scheme_id,
        number: file_id
    }).ok_or(Error::TooManyFiles)
}

/// Close syscall
pub fn close(fd: usize) -> Result<usize> {
    println!("Close {}", fd);
    Ok(0)
}
