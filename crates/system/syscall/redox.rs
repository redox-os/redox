use super::syscall;

pub const SYS_DEBUG: usize = 0;

pub const SYS_ALLOC: usize = 1000;
pub const SYS_REALLOC: usize = 1001;
pub const SYS_REALLOC_INPLACE: usize = 1002;
pub const SYS_UNALLOC: usize = 1003;

#[no_mangle]
pub unsafe fn sys_debug(ptr: *const u8, len: usize) -> usize {
    syscall(SYS_DEBUG, ptr as usize, len, 0)
}

#[no_mangle]
pub unsafe fn sys_alloc(size: usize) -> usize {
    syscall(SYS_ALLOC, size, 0, 0)
}

#[no_mangle]
pub unsafe fn sys_realloc(ptr: usize, size: usize) -> usize {
    syscall(SYS_REALLOC, ptr, size, 0)
}

#[no_mangle]
pub unsafe fn sys_realloc_inplace(ptr: usize, size: usize) -> usize {
    syscall(SYS_REALLOC_INPLACE, ptr, size, 0)
}

#[no_mangle]
pub unsafe fn sys_unalloc(ptr: usize) -> usize {
    syscall(SYS_UNALLOC, ptr, 0, 0)
}
