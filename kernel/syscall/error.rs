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
