use super::{Error, Result};

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
    /// Wait for a process
    WaitPid = 7,
    /// Execute syscall
    Exec = 11,
    /// Change working directory
    ChDir = 12,
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
    SchedYield = 158,
    /// Get process working directory
    GetCwd = 183
}

/// Convert numbers to calls
/// See http://syscalls.kernelgrok.com/
impl Call {
    //TODO: Return Option<Call>
    pub fn from(number: usize) -> Result<Call> {
        match number {
            1 => Ok(Call::Exit),
            3 => Ok(Call::Read),
            4 => Ok(Call::Write),
            5 => Ok(Call::Open),
            6 => Ok(Call::Close),
            7 => Ok(Call::WaitPid),
            11 => Ok(Call::Exec),
            12 => Ok(Call::ChDir),
            20 => Ok(Call::GetPid),
            41 => Ok(Call::Dup),
            45 => Ok(Call::Brk),
            110 => Ok(Call::Iopl),
            120 => Ok(Call::Clone),
            158 => Ok(Call::SchedYield),
            183 => Ok(Call::GetCwd),
            _ => Err(Error::NoCall)
        }
    }
}
