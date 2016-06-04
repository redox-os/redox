use syscall::arch::{syscall0, syscall1, syscall2, syscall3};
use error::Result;

pub const SYS_BRK: usize = 45;
pub const SYS_CHDIR: usize = 12;
pub const SYS_CLONE: usize = 120;
    pub const CLONE_VM: usize = 0x100;
    pub const CLONE_FS: usize = 0x200;
    pub const CLONE_FILES: usize = 0x400;
    pub const CLONE_VFORK: usize = 0x4000;
    /// Mark this clone as supervised.
    ///
    /// This means that the process can run in supervised mode, even not being connected to
    /// a supervisor yet. In other words, the parent can later on supervise the process and handle
    /// the potential blocking syscall.
    ///
    /// This is an important security measure, since otherwise the process would be able to fork it
    /// self right after starting, making supervising it impossible.
    pub const CLONE_SUPERVISE: usize = 0x400000;
pub const SYS_CLOSE: usize = 6;
pub const SYS_CLOCK_GETTIME: usize = 265;
    pub const CLOCK_REALTIME: usize = 1;
    pub const CLOCK_MONOTONIC: usize = 4;
pub const SYS_DUP: usize = 41;
pub const SYS_EXECVE: usize = 11;
pub const SYS_EXIT: usize = 1;
pub const SYS_FPATH: usize = 928;
pub const SYS_FSTAT: usize = 28;
pub const SYS_FSYNC: usize = 118;
pub const SYS_FTRUNCATE: usize = 93;
pub const SYS_GETPID: usize = 20;
pub const SYS_IOPL: usize = 110;
pub const SYS_LINK: usize = 9;
pub const SYS_LSEEK: usize = 19;
    pub const SEEK_SET: usize = 0;
    pub const SEEK_CUR: usize = 1;
    pub const SEEK_END: usize = 2;
pub const SYS_MKDIR: usize = 39;
pub const SYS_NANOSLEEP: usize = 162;
pub const SYS_OPEN: usize = 5;
    pub const O_RDONLY: usize = 0;
    pub const O_WRONLY: usize = 1;
    pub const O_RDWR: usize = 2;
    pub const O_NONBLOCK: usize = 4;
    pub const O_APPEND: usize = 8;
    pub const O_SHLOCK: usize = 0x10;
    pub const O_EXLOCK: usize = 0x20;
    pub const O_ASYNC: usize = 0x40;
    pub const O_FSYNC: usize = 0x80;
    pub const O_CREAT: usize = 0x200;
    pub const O_TRUNC: usize = 0x400;
    pub const O_EXCL: usize = 0x800;
pub const SYS_PIPE2: usize = 331;
pub const SYS_READ: usize = 3;
pub const SYS_RMDIR: usize = 84;
pub const SYS_STAT: usize = 18;
    pub const MODE_DIR: u16 = 0x4000;
    pub const MODE_FILE: u16 = 0x8000;
pub const SYS_UNLINK: usize = 10;
pub const SYS_WAITPID: usize = 7;
pub const SYS_WRITE: usize = 4;
pub const SYS_YIELD: usize = 158;

#[derive(Copy, Clone, Debug, Default)]
#[repr(packed)]
pub struct Stat {
    pub st_dev: u16,
    pub st_ino: u16,
    pub st_mode: u16,
    pub st_nlink: u16,
    pub st_uid: u16,
    pub st_gid: u16,
    pub st_rdev: u16,
    pub st_size: u32,
    pub st_atime: u32,
    pub st_mtime: u32,
    pub st_ctime: u32
}

#[derive(Copy, Clone, Debug, Default)]
#[repr(packed)]
pub struct TimeSpec {
    pub tv_sec: i64,
    pub tv_nsec: i32,
}

pub unsafe fn sys_brk(addr: usize) -> Result<usize> {
    syscall1(SYS_BRK, addr)
}

pub unsafe fn sys_chdir(path: *const u8) -> Result<usize> {
    syscall1(SYS_CHDIR, path as usize)
}

pub unsafe fn sys_clone(flags: usize) -> Result<usize> {
    syscall1(SYS_CLONE, flags)
}

pub fn sys_close(fd: usize) -> Result<usize> {
    unsafe { syscall1(SYS_CLOSE, fd) }
}

pub fn sys_clock_gettime(clock: usize, tp: &mut TimeSpec) -> Result<usize> {
    unsafe { syscall2(SYS_CLOCK_GETTIME, clock, tp as *mut TimeSpec as usize) }
}

pub fn sys_dup(fd: usize) -> Result<usize> {
    unsafe { syscall1(SYS_DUP, fd) }
}

pub unsafe fn sys_execve(path: *const u8, args: *const *const u8) -> Result<usize> {
    syscall2(SYS_EXECVE, path as usize, args as usize)
}

pub fn sys_exit(status: usize) -> Result<usize> {
    unsafe { syscall1(SYS_EXIT, status) }
}

pub fn sys_fpath(fd: usize, buf: &mut [u8]) -> Result<usize> {
    unsafe { syscall3(SYS_FPATH, fd, buf.as_mut_ptr() as usize, buf.len()) }
}

pub fn sys_fstat(fd: usize, stat: &mut Stat) -> Result<usize> {
    unsafe { syscall2(SYS_FSTAT, fd, stat as *mut Stat as usize) }
}

pub fn sys_fsync(fd: usize) -> Result<usize> {
    unsafe { syscall1(SYS_FSYNC, fd) }
}

pub fn sys_ftruncate(fd: usize, len: usize) -> Result<usize> {
    unsafe { syscall2(SYS_FTRUNCATE, fd, len) }
}

pub fn sys_getpid() -> Result<usize> {
    unsafe { syscall0(SYS_GETPID) }
}

pub unsafe fn sys_iopl(level: usize) -> Result<usize> {
    syscall1(SYS_IOPL, level)
}

pub unsafe fn sys_link(old: *const u8, new: *const u8) -> Result<usize> {
    syscall2(SYS_LINK, old as usize, new as usize)
}

pub fn sys_lseek(fd: usize, offset: isize, whence: usize) -> Result<usize> {
    unsafe { syscall3(SYS_LSEEK, fd, offset as usize, whence) }
}

pub unsafe fn sys_mkdir(path: *const u8, mode: usize) -> Result<usize> {
    syscall2(SYS_MKDIR, path as usize, mode)
}

pub fn sys_nanosleep(req: &TimeSpec, rem: &mut TimeSpec) -> Result<usize> {
    unsafe { syscall2(SYS_NANOSLEEP, req as *const TimeSpec as usize, rem as *mut TimeSpec as usize) }
}

pub unsafe fn sys_open(path: *const u8, flags: usize, mode: usize) -> Result<usize> {
    syscall3(SYS_OPEN, path as usize, flags, mode)
}

pub unsafe fn sys_pipe2(fds: *mut usize, flags: usize) -> Result<usize> {
    syscall2(SYS_PIPE2, fds as usize, flags)
}

pub fn sys_read(fd: usize, buf: &mut [u8]) -> Result<usize> {
    unsafe { syscall3(SYS_READ, fd, buf.as_mut_ptr() as usize, buf.len()) }
}

pub unsafe fn sys_rmdir(path: *const u8) -> Result<usize> {
    syscall1(SYS_RMDIR, path as usize)
}

pub unsafe fn sys_stat(path: *const u8, stat: &mut Stat) -> Result<usize> {
    syscall2(SYS_STAT, path as usize, stat as *mut Stat as usize)
}

pub unsafe fn sys_unlink(path: *const u8) -> Result<usize> {
    syscall1(SYS_UNLINK, path as usize)
}

pub fn sys_waitpid(pid: usize, status: &mut usize, options: usize) -> Result<usize> {
    unsafe { syscall3(SYS_WAITPID, pid, status as *mut usize as usize, options) }
}

pub fn sys_write(fd: usize, buf: &[u8]) -> Result<usize> {
    unsafe { syscall3(SYS_WRITE, fd, buf.as_ptr() as usize, buf.len()) }
}

pub fn sys_yield() -> Result<usize> {
    unsafe { syscall0(SYS_YIELD) }
}
