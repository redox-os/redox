///! Syscall handlers

use core::slice;

pub use self::fs::*;
pub use self::process::*;

/// Filesystem syscalls
pub mod fs;

/// Process syscalls
pub mod process;

/// System call list
#[derive(Copy, Clone, Debug)]
pub enum Call {
    /// Exit syscall
    Exit,
    /// Read syscall
    Read,
    /// Write syscall
    Write,
    /// Open syscall
    Open,
    /// Close syscall
    Close,
    /// Unknown syscall
    Unknown
}

/// Convert numbers to calls
/// See http://syscalls.kernelgrok.com/
impl From<usize> for Call {
    fn from(number: usize) -> Call {
        match number {
            1 => Call::Exit,
            3 => Call::Read,
            4 => Call::Write,
            5 => Call::Open,
            6 => Call::Close,
            _ => Call::Unknown
        }
    }
}

/// The error number for an invalid value
#[derive(Copy, Clone, Debug)]
pub enum Error {
    /// Operation not permitted
    NotPermitted,
    /// No such file or directory
    NoEntry,
    /// Invalid argument
    InvalidValue,
    /// Syscall not implemented
    NoCall
}

/// Convert errors to numbers
/// See http://www-numi.fnal.gov/offline_software/srt_public_context/WebDocs/Errors/unix_system_errors.html
impl From<Error> for usize {
    fn from(err: Error) -> usize {
        match err {
            Error::NotPermitted => 1,
            Error::NoEntry => 2,
            Error::InvalidValue => 22,
            Error::NoCall => 38
        }
    }
}

pub type Result<T> = ::core::result::Result<T, Error>;

/// Convert a pointer and length to slice, if valid
/// TODO: Check validity
pub fn convert_slice(ptr: usize, len: usize) -> Result<&'static [u8]> {
    Ok(unsafe { slice::from_raw_parts(ptr as *const u8, len) })
}

/// Convert a pointer and length to slice, if valid
/// TODO: Check validity
pub fn convert_slice_mut(ptr: usize, len: usize) -> Result<&'static mut [u8]> {
    Ok(unsafe { slice::from_raw_parts_mut(ptr as *mut u8, len) })
}

pub fn handle(a: usize, b: usize, c: usize, d: usize) -> ::core::result::Result<usize, usize> {
    match Call::from(a) {
        Call::Exit => exit(b),
        Call::Read => read(b, convert_slice_mut(c, d)?),
        Call::Write => write(b, convert_slice(c, d)?),
        Call::Open => open(convert_slice(b, c)?, d),
        Call::Close => close(b),
        Call::Unknown => Err(Error::NoCall)
    }.map_err(|err| err.into())
}
