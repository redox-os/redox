use super::syscall;

pub const SYS_BRK: usize = 45;
pub const SYS_CHDIR: usize = 12;
pub const SYS_CLOSE: usize = 6;
pub const SYS_CLONE: usize = 120;
    pub const CLONE_VM: usize = 0x100;
    pub const CLONE_FS: usize = 0x200;
    pub const CLONE_FILES: usize = 0x400;
    pub const CLONE_VFORK: usize = 0x4000;
pub const SYS_CLOCK_GETTIME: usize = 265;
    pub const CLOCK_REALTIME: usize = 1;
    pub const CLOCK_MONOTONIC: usize = 4;
pub const SYS_DUP: usize = 41;
pub const SYS_EXECVE: usize = 11;
pub const SYS_SPAWNVE: usize = 3011; //Extra to fix scheme execve
pub const SYS_EXIT: usize = 1;
pub const SYS_FPATH: usize = 3001;
pub const SYS_FSTAT: usize = 28;
pub const SYS_FSYNC: usize = 118;
pub const SYS_FTRUNCATE: usize = 93;
pub const SYS_GETPID: usize = 20;
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
pub const SYS_UNLINK: usize = 10;
pub const SYS_WAITPID: usize = 7;
pub const SYS_WRITE: usize = 4;
pub const SYS_YIELD: usize = 158;

#[repr(packed)]
pub struct TimeSpec {
    pub tv_sec: i64,
    pub tv_nsec: i32,
}

#[no_mangle]
pub unsafe fn sys_brk(addr: usize) -> usize {
    syscall(SYS_BRK, addr, 0, 0)
}

#[no_mangle]
pub unsafe fn sys_chdir(path: *const u8) -> usize {
    syscall(SYS_CHDIR, path as usize, 0, 0)
}

#[no_mangle]
pub unsafe fn sys_clone(flags: usize) -> usize {
    syscall(SYS_CLONE, flags, 0, 0)
}

#[no_mangle]
pub unsafe fn sys_close(fd: usize) -> usize {
    syscall(SYS_CLOSE, fd, 0, 0)
}

#[no_mangle]
pub unsafe fn sys_clock_gettime(clock: usize, tp: *mut TimeSpec) -> usize {
    syscall(SYS_CLOCK_GETTIME, clock, tp as usize, 0)
}

#[no_mangle]
pub unsafe fn sys_dup(fd: usize) -> usize {
    syscall(SYS_DUP, fd, 0, 0)
}

#[no_mangle]
pub unsafe fn sys_execve(path: *const u8, args: *const *const u8) -> usize {
    syscall(SYS_EXECVE, path as usize, args as usize, 0)
}

#[no_mangle]
pub unsafe fn sys_spawnve(path: *const u8, args: *const *const u8) -> usize {
    syscall(SYS_SPAWNVE, path as usize, args as usize, 0)
}

#[no_mangle]
pub unsafe fn sys_exit(status: isize) {
    syscall(SYS_EXIT, status as usize, 0, 0);
}

#[no_mangle]
pub unsafe fn sys_fpath(fd: usize, buf: *mut u8, len: usize) -> usize {
    syscall(SYS_FPATH, fd, buf as usize, len)
}

// TODO: FSTAT

#[no_mangle]
pub unsafe fn sys_fsync(fd: usize) -> usize {
    syscall(SYS_FSYNC, fd, 0, 0)
}

#[no_mangle]
pub unsafe fn sys_ftruncate(fd: usize, len: usize) -> usize {
    syscall(SYS_FTRUNCATE, fd, len, 0)
}

#[no_mangle]
pub unsafe fn sys_getpid() -> usize {
    syscall(SYS_GETPID, 0, 0, 0)
}

#[no_mangle]
pub unsafe fn sys_link(old: *const u8, new: *const u8) -> usize {
    syscall(SYS_LINK, old as usize, new as usize, 0)
}

#[no_mangle]
pub unsafe fn sys_lseek(fd: usize, offset: isize, whence: usize) -> usize {
    syscall(SYS_LSEEK, fd, offset as usize, whence)
}

#[no_mangle]
pub unsafe fn sys_mkdir(path: *const u8, mode: usize) -> usize {
    syscall(SYS_MKDIR, path as usize, 0, mode)
}

#[no_mangle]
pub unsafe fn sys_nanosleep(req: *const TimeSpec, rem: *mut TimeSpec) -> usize {
    syscall(SYS_NANOSLEEP, req as usize, rem as usize, 0)
}

#[no_mangle]
pub unsafe fn sys_open(path: *const u8, flags: usize, mode: usize) -> usize {
    syscall(SYS_OPEN, path as usize, flags, mode)
}

#[no_mangle]
pub unsafe fn sys_pipe2(fds: *mut usize, flags: usize) -> usize {
    syscall(SYS_PIPE2, fds as usize, flags, 0)
}

#[no_mangle]
pub unsafe fn sys_read(fd: usize, buf: *mut u8, count: usize) -> usize {
    syscall(SYS_READ, fd, buf as usize, count)
}

#[no_mangle]
pub unsafe fn sys_unlink(path: *const u8) -> usize {
    syscall(SYS_UNLINK, path as usize, 0, 0)
}

#[no_mangle]
pub unsafe fn sys_waitpid(pid: isize, status: *mut usize, options: usize) -> usize {
    syscall(SYS_WAITPID, pid as usize, status as usize, options)
}

#[no_mangle]
pub unsafe fn sys_write(fd: usize, buf: *const u8, count: usize) -> usize {
    syscall(SYS_WRITE, fd, buf as usize, count)
}

#[no_mangle]
pub unsafe fn sys_yield() -> usize {
    syscall(SYS_YIELD, 0, 0, 0)
}
