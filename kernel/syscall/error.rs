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
    /// Bad address
    Fault = 14,
    /// File exists
    FileExists = 17,
    /// No such device
    NoDevice = 19,
    /// Invalid argument
    InvalidValue = 22,
    /// Too many open files
    TooManyFiles = 24,
    /// Illegal seek
    IllegalSeek = 29,
    /// Syscall not implemented
    NoCall = 38
}

impl Error {
    pub fn from(number: usize) -> Option<Error> {
        match number {
            1 => Some(Error::NotPermitted),
            2 => Some(Error::NoEntry),
            3 => Some(Error::NoProcess),
            8 => Some(Error::NoExec),
            9 => Some(Error::BadFile),
            11 => Some(Error::TryAgain),
            14 => Some(Error::Fault),
            17 => Some(Error::FileExists),
            19 => Some(Error::NoDevice),
            22 => Some(Error::InvalidValue),
            24 => Some(Error::TooManyFiles),
            29 => Some(Error::IllegalSeek),
            38 => Some(Error::NoCall),
            _ => None
        }
    }
}

pub type Result<T> = ::core::result::Result<T, Error>;

pub fn convert_to_result(number: usize) -> Result<usize> {
    if let Some(err) = Error::from((-(number as isize)) as usize) {
        Err(err)
    } else {
        Ok(number)
    }
}
