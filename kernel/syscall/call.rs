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
    /// Sync file descriptor
    FSync = 118,
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
    pub fn from(number: usize) -> Option<Call> {
        match number {
            1 => Some(Call::Exit),
            3 => Some(Call::Read),
            4 => Some(Call::Write),
            5 => Some(Call::Open),
            6 => Some(Call::Close),
            7 => Some(Call::WaitPid),
            11 => Some(Call::Exec),
            12 => Some(Call::ChDir),
            20 => Some(Call::GetPid),
            41 => Some(Call::Dup),
            45 => Some(Call::Brk),
            110 => Some(Call::Iopl),
            118 => Some(Call::FSync),
            120 => Some(Call::Clone),
            158 => Some(Call::SchedYield),
            183 => Some(Call::GetCwd),
            _ => None
        }
    }
}
