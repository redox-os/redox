use syscall::arch::{syscall2, syscall1};
use error::Result;

pub const SYS_DEBUG: usize = 0;
pub const SYS_SUPERVISE: usize = 1638; // loominatzi confirmed

pub fn sys_debug(buf: &[u8]) -> Result<usize> {
    unsafe { syscall2(SYS_DEBUG, buf.as_ptr() as usize, buf.len()) }
}

/// <!-- @MANSTART{supervise} -->
/// Supervise a given child process' system calls.
///
/// SUPERVISE allows a process to run another process in a restricted, traced, and supervised
/// environment, which is useful for various purposes, such as emulation, virtualisation, tracing,
/// logging, and debugging.
///
/// SUPERVISE takes a PID specifing the process to be supervised. This PID must be a child process
/// of the invoker. If not, EACCES will be returned.
///
/// A process can only have one supervisor at a time. If SUPERVISE is called on a process, which
/// already have a supervisor EPERM will be returned.
///
/// The process identified by the given PID will be restricted in such a way, that every syscall
/// made will mark the process as blocked and store the syscall until it is handled by the parrent.
///
/// The return value (if successful) is a file descriptor, from which syscalls can be read and written:
/// the syscalls are read in `Packet` sized packages, containing the respective blocking syscall. If
/// no syscall is blocking (or the last blocking syscall have been handled), 0 bytes will be read to
/// the buffer.
///
/// Writing pointer sized integers to this file handle will set the EAX register of the particular
/// process, after which the process is unblocked and the syscall buffer is emptied. The behavior of
/// writing packages of unexpected size is unspecified.
///
/// Note that a process blocked by a syscall will have its potential sleep cleared (i.e., it will
/// not wake up after the sleep is finished).
///
/// Passing a non-existent PID results in ESRCH.
///
/// A process being supervised is referred to as 'jailed' or 'supervised'.
/// <!-- @MANEND -->
pub fn sys_supervise(pid: usize) -> Result<usize> {
    unsafe { syscall1(SYS_SUPERVISE, pid) }
}
