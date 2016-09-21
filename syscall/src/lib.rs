#![feature(asm)]
#![no_std]

pub use self::arch::*;
pub use self::data::*;
pub use self::error::*;
pub use self::flag::*;
pub use self::number::*;
pub use self::scheme::*;

#[cfg(target_arch = "x86")]
#[path="x86.rs"]
mod arch;

#[cfg(target_arch = "x86_64")]
#[path="x86_64.rs"]
mod arch;

pub mod data;

pub mod error;

pub mod flag;

pub mod number;

pub mod scheme;

pub unsafe fn brk(addr: usize) -> Result<usize> {
    syscall1(SYS_BRK, addr)
}

pub fn chdir(path: &str) -> Result<usize> {
    unsafe { syscall2(SYS_CHDIR, path.as_ptr() as usize, path.len()) }
}

pub unsafe fn clone(flags: usize) -> Result<usize> {
    syscall1_clobber(SYS_CLONE, flags)
}

pub fn close(fd: usize) -> Result<usize> {
    unsafe { syscall1(SYS_CLOSE, fd) }
}

pub fn clock_gettime(clock: usize, tp: &mut TimeSpec) -> Result<usize> {
    unsafe { syscall2(SYS_CLOCK_GETTIME, clock, tp as *mut TimeSpec as usize) }
}

pub fn dup(fd: usize) -> Result<usize> {
    unsafe { syscall1(SYS_DUP, fd) }
}

pub fn execve(path: &str, args: &[[usize; 2]]) -> Result<usize> {
    unsafe { syscall4(SYS_EXECVE, path.as_ptr() as usize, path.len(), args.as_ptr() as usize, args.len()) }
}

pub fn exit(status: usize) -> Result<usize> {
    unsafe { syscall1(SYS_EXIT, status) }
}

pub fn fpath(fd: usize, buf: &mut [u8]) -> Result<usize> {
    unsafe { syscall3(SYS_FPATH, fd, buf.as_mut_ptr() as usize, buf.len()) }
}

pub fn fstat(fd: usize, stat: &mut Stat) -> Result<usize> {
    unsafe { syscall2(SYS_FSTAT, fd, stat as *mut Stat as usize) }
}

pub fn fsync(fd: usize) -> Result<usize> {
    unsafe { syscall1(SYS_FSYNC, fd) }
}

pub fn ftruncate(fd: usize, len: usize) -> Result<usize> {
    unsafe { syscall2(SYS_FTRUNCATE, fd, len) }
}

pub unsafe fn futex(addr: *mut i32, op: usize, val: i32, val2: usize, addr2: *mut i32) -> Result<usize> {
    syscall5(SYS_FUTEX, addr as usize, op, (val as isize) as usize, val2, addr2 as usize)
}

pub fn getcwd(buf: &mut [u8]) -> Result<usize> {
    unsafe { syscall2(SYS_GETCWD, buf.as_mut_ptr() as usize, buf.len()) }
}

pub fn getpid() -> Result<usize> {
    unsafe { syscall0(SYS_GETPID) }
}

pub unsafe fn iopl(level: usize) -> Result<usize> {
    syscall1(SYS_IOPL, level)
}

pub unsafe fn link(old: *const u8, new: *const u8) -> Result<usize> {
    syscall2(SYS_LINK, old as usize, new as usize)
}

pub fn lseek(fd: usize, offset: isize, whence: usize) -> Result<usize> {
    unsafe { syscall3(SYS_LSEEK, fd, offset as usize, whence) }
}

pub fn mkdir(path: &str, mode: usize) -> Result<usize> {
    unsafe { syscall3(SYS_MKDIR, path.as_ptr() as usize, path.len(), mode) }
}

pub fn nanosleep(req: &TimeSpec, rem: &mut TimeSpec) -> Result<usize> {
    unsafe { syscall2(SYS_NANOSLEEP, req as *const TimeSpec as usize, rem as *mut TimeSpec as usize) }
}

pub fn open(path: &str, flags: usize) -> Result<usize> {
    unsafe { syscall3(SYS_OPEN, path.as_ptr() as usize, path.len(), flags) }
}

pub unsafe fn physmap(physical_address: usize, size: usize, flags: usize) -> Result<usize> {
    syscall3(SYS_PHYSMAP, physical_address, size, flags)
}

pub unsafe fn physunmap(virtual_address: usize) -> Result<usize> {
    syscall1(SYS_PHYSUNMAP, virtual_address)
}

pub fn pipe2(fds: &mut [usize; 2], flags: usize) -> Result<usize> {
    unsafe { syscall2(SYS_PIPE2, fds.as_ptr() as usize, flags) }
}

pub fn read(fd: usize, buf: &mut [u8]) -> Result<usize> {
    unsafe { syscall3(SYS_READ, fd, buf.as_mut_ptr() as usize, buf.len()) }
}

pub fn rmdir(path: &str) -> Result<usize> {
    unsafe { syscall2(SYS_RMDIR, path.as_ptr() as usize, path.len()) }
}

pub fn unlink(path: &str) -> Result<usize> {
    unsafe { syscall2(SYS_UNLINK, path.as_ptr() as usize, path.len()) }
}

pub fn waitpid(pid: usize, status: &mut usize, options: usize) -> Result<usize> {
    unsafe { syscall3(SYS_WAITPID, pid, status as *mut usize as usize, options) }
}

pub fn write(fd: usize, buf: &[u8]) -> Result<usize> {
    unsafe { syscall3(SYS_WRITE, fd, buf.as_ptr() as usize, buf.len()) }
}

pub fn sched_yield() -> Result<usize> {
    unsafe { syscall0(SYS_YIELD) }
}
