///! Syscall handlers

use core::slice;

pub use self::fs::*;
pub use self::process::*;

/// Filesystem syscalls
pub mod fs;

/// Process syscalls
pub mod process;

/// System call list
/// See http://syscalls.kernelgrok.com/ for numbers
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum Call {
    /// Exit syscall
    Exit = 1,
    /// Read syscall
    Read = 3,
    /// Write syscall
    Write = 4,
    /// Open syscall
    Open = 5,
    /// Close syscall
    Close = 6,
    /// Execute syscall
    Exec = 11,
    /// Get process ID
    GetPid = 20,
    /// Duplicate file descriptor
    Dup = 41,
    /// Set process break
    Brk = 45,
    /// Set process I/O privilege level
    Iopl = 110,
    /// Clone process
    Clone = 120,
    /// Yield to scheduler
    SchedYield = 158
}

/// Convert numbers to calls
/// See http://syscalls.kernelgrok.com/
impl Call {
    fn from(number: usize) -> Result<Call> {
        match number {
            1 => Ok(Call::Exit),
            3 => Ok(Call::Read),
            4 => Ok(Call::Write),
            5 => Ok(Call::Open),
            6 => Ok(Call::Close),
            11 => Ok(Call::Exec),
            20 => Ok(Call::GetPid),
            41 => Ok(Call::Dup),
            45 => Ok(Call::Brk),
            110 => Ok(Call::Iopl),
            120 => Ok(Call::Clone),
            158 => Ok(Call::SchedYield),
            _ => Err(Error::NoCall)
        }
    }
}

/// The error number for an invalid value
/// See http://www-numi.fnal.gov/offline_software/srt_public_context/WebDocs/Errors/unix_system_errors.html for numbers
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum Error {
    /// Operation not permitted
    NotPermitted = 1,
    /// No such file or directory
    NoEntry = 2,
    /// No such process
    NoProcess = 3,
    /// Invalid executable format
    NoExec = 8,
    /// Bad file number
    BadFile = 9,
    /// Try again
    TryAgain = 11,
    /// File exists
    FileExists = 17,
    /// Invalid argument
    InvalidValue = 22,
    /// Too many open files
    TooManyFiles = 24,
    /// Syscall not implemented
    NoCall = 38
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

pub fn handle(a: usize, b: usize, c: usize, d: usize, e: usize, _f: usize) -> Result<usize> {
    match Call::from(a) {
        Ok(call) => match call {
            Call::Exit => exit(b),
            Call::Read => read(b, convert_slice_mut(c as *mut u8, d)?),
            Call::Write => write(b, convert_slice(c as *const u8, d)?),
            Call::Open => open(convert_slice(b as *const u8, c)?, d),
            Call::Close => close(b),
            Call::Exec => exec(convert_slice(b as *const u8, c)?, convert_slice(d as *const [usize; 2], e)?),
            Call::GetPid => getpid(),
            Call::Dup => dup(b),
            Call::Brk => brk(b),
            Call::Iopl => iopl(b),
            Call::Clone => clone(b),
            Call::SchedYield => sched_yield()
        },
        Err(err) => {
            println!("Unknown syscall {}", a);
            Err(err)
        }
    }
}

#[no_mangle]
pub extern fn syscall(a: usize, b: usize, c: usize, d: usize, e: usize, f: usize) -> usize {
    match handle(a, b, c, d, e, f) {
        Ok(value) => value,
        Err(value) => (-(value as isize)) as usize
    }
}
