use syscall::arch::{syscall1, syscall2};
use error::Result;

pub const SYS_DEBUG: usize = 0;

pub const SYS_ALLOC: usize = 1000;
pub const SYS_REALLOC: usize = 1001;
pub const SYS_REALLOC_INPLACE: usize = 1002;
pub const SYS_UNALLOC: usize = 1003;

pub fn sys_debug(buf: &[u8]) -> Result<usize> {
    unsafe { syscall2(SYS_DEBUG, buf.as_ptr() as usize, buf.len()) }
}

pub unsafe fn sys_alloc(size: usize) -> Result<usize> {
    syscall1(SYS_ALLOC, size)
}

pub unsafe fn sys_realloc(ptr: usize, size: usize) -> Result<usize> {
    syscall2(SYS_REALLOC, ptr, size)
}

pub unsafe fn sys_realloc_inplace(ptr: usize, size: usize) -> Result<usize> {
    syscall2(SYS_REALLOC_INPLACE, ptr, size)
}

pub unsafe fn sys_unalloc(ptr: usize) -> Result<usize> {
    syscall1(SYS_UNALLOC, ptr)
}
