use syscall::common::*;

#[path="../../kernel/syscall/common.rs"]
pub mod common;

#[cold]
#[inline(never)]
#[cfg(target_arch = "x86")]
pub unsafe fn syscall(mut a: usize, b: usize, c: usize, d: usize) -> usize {
    asm!("int 0x80"
        : "={eax}"(a)
        : "{eax}"(a), "{ebx}"(b), "{ecx}"(c), "{edx}"(d)
        : "memory"
        : "intel", "volatile");

    a
}

#[cold]
#[inline(never)]
#[cfg(target_arch = "x86_64")]
pub unsafe fn syscall(mut a: usize, b: usize, c: usize, d: usize) -> usize {
    asm!("int 0x80"
        : "={rax}"(a)
        : "{rax}"(a), "{rbx}"(b), "{rcx}"(c), "{rdx}"(d)
        : "memory"
        : "intel", "volatile");

    a
}

pub unsafe fn sys_debug(byte: u8) {
    syscall(SYS_DEBUG, byte as usize, 0, 0);
}

pub unsafe fn sys_brk(addr: usize) -> usize {
    syscall(SYS_BRK, addr, 0, 0)
}

pub unsafe fn sys_chdir(path: *const u8) -> usize {
    syscall(SYS_CHDIR, path as usize, 0, 0)
}

pub unsafe fn sys_close(fd: usize) -> usize {
    syscall(SYS_CLOSE, fd, 0, 0)
}

pub unsafe fn sys_dup(fd: usize) -> usize {
    syscall(SYS_DUP, fd, 0, 0)
}

pub unsafe fn sys_execve(path: *const u8) -> usize {
    syscall(SYS_EXECVE, path as usize, 0, 0)
}

pub unsafe fn sys_exit(status: isize) {
    syscall(SYS_EXIT, status as usize, 0, 0);
}

pub unsafe fn sys_fork() -> usize {
    syscall(SYS_FORK, 0, 0, 0)
}

pub unsafe fn sys_fpath(fd: usize, buf: *mut u8, len: usize) -> usize {
    syscall(SYS_FPATH, fd, buf as usize, len)
}

//TODO: FSTAT

pub unsafe fn sys_fsync(fd: usize) -> usize {
    syscall(SYS_FSYNC, fd, 0, 0)
}

pub unsafe fn sys_ftruncate(fd: usize, len: usize) -> usize {
    syscall(SYS_FTRUNCATE, fd, len, 0)
}

#[repr(packed)]
pub struct TV {
    pub tv_sec: i64,
    pub tv_usec: i32,
}

//TODO: gettimeofday
pub unsafe fn sys_gettimeofday(tv: *mut TV) -> usize{
    syscall(SYS_GETTIMEOFDAY, tv as usize, 0, 0)
}

pub unsafe fn sys_link(old: *const u8, new: *const u8) -> usize {
    syscall(SYS_LINK, old as usize, new as usize, 0)
}

pub unsafe fn sys_lseek(fd: usize, offset: isize, whence: usize) -> usize {
    syscall(SYS_LSEEK, fd, offset as usize, whence)
}

pub unsafe fn sys_open(path: *const u8, flags: usize, mode: usize) -> usize {
    syscall(SYS_OPEN, path as usize, flags, mode)
}

pub unsafe fn sys_read(fd: usize, buf: *mut u8, count: usize) -> usize {
    syscall(SYS_READ, fd, buf as usize, count)
}

pub unsafe fn sys_unlink(path: *const u8) -> usize {
    syscall(SYS_UNLINK, path as usize, 0, 0)
}

pub unsafe fn sys_write(fd: usize, buf: *const u8, count: usize) -> usize {
    syscall(SYS_WRITE, fd, buf as usize, count)
}

pub unsafe fn sys_yield() {
    syscall(SYS_YIELD, 0, 0, 0);
}

pub unsafe fn sys_alloc(size: usize) -> usize {
    syscall(SYS_ALLOC, size, 0, 0)
}

pub unsafe fn sys_realloc(ptr: usize, size: usize) -> usize {
    syscall(SYS_REALLOC, ptr, size, 0)
}

pub unsafe fn sys_realloc_inplace(ptr: usize, size: usize) -> usize {
    syscall(SYS_REALLOC_INPLACE, ptr, size, 0)
}

pub unsafe fn sys_unalloc(ptr: usize) {
    syscall(SYS_UNALLOC, ptr, 0, 0);
}
