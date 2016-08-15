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
    /// Execute syscall
    Exec,
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
            11 => Call::Exec,
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
    /// Bad file number
    BadFile,
    /// Invalid argument
    InvalidValue,
    /// Too many open files
    TooManyFiles,
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
            Error::BadFile => 9,
            Error::InvalidValue => 22,
            Error::TooManyFiles => 24,
            Error::NoCall => 38
        }
    }
}

pub type Result<T> = ::core::result::Result<T, Error>;

/// Convert a pointer and length to slice, if valid
/// TODO: Check validity
pub fn convert_slice<T>(ptr: *const T, len: usize) -> Result<&'static [T]> {
    Ok(unsafe { slice::from_raw_parts(ptr, len) })
}

/// Convert a pointer and length to slice, if valid
/// TODO: Check validity
pub fn convert_slice_mut<T>(ptr: *mut T, len: usize) -> Result<&'static mut [T]> {
    Ok(unsafe { slice::from_raw_parts_mut(ptr, len) })
}

pub fn handle(a: usize, b: usize, c: usize, d: usize, e: usize, f: usize) -> ::core::result::Result<usize, usize> {
    match Call::from(a) {
        Call::Exit => exit(b),
        Call::Read => read(b, convert_slice_mut(c as *mut u8, d)?),
        Call::Write => write(b, convert_slice(c as *const u8, d)?),
        Call::Open => open(convert_slice(b as *const u8, c)?, d),
        Call::Close => close(b),
        Call::Exec => exec(convert_slice(b as *const u8, c)?, convert_slice(d as *const [usize; 2], e)?),
        Call::Unknown => Err(Error::NoCall)
    }.map_err(|err| err.into())
}
