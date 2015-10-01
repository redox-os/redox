use common::event::*;
use common::time::*;

use graphics::window::*;

use syscall::common::*;

pub unsafe fn syscall(mut eax: u32, ebx: u32, ecx: u32, edx: u32) -> u32 {
    asm!("int 0x80"
        : "={eax}"(eax)
        : "{eax}"(eax), "{ebx}"(ebx), "{ecx}"(ecx), "{edx}"(edx)
        : "memory"
        : "intel", "volatile");

    eax
}

pub unsafe fn sys_debug(byte: u8) {
    syscall(SYS_DEBUG, byte as u32, 0, 0);
}

pub unsafe fn sys_exit(status: isize) {
    syscall(SYS_EXIT, (status as i32) as u32, 0, 0);
}

pub unsafe fn sys_read(fd: usize, buf: *mut u8, count: usize) -> usize {
    syscall(SYS_READ, fd as u32, buf as u32, count as u32) as usize
}

pub unsafe fn sys_write(fd: usize, buf: *const u8, count: usize) -> usize {
    syscall(SYS_WRITE, fd as u32, buf as u32, count as u32) as usize
}

pub unsafe fn sys_open(path: *const u8, flags: isize, mode: isize) -> usize {
    syscall(SYS_OPEN, path as u32, (flags as i32) as u32, (mode as i32) as u32) as usize
}

pub unsafe fn sys_close(fd: usize) -> usize {
    syscall(SYS_CLOSE, fd as u32, 0, 0) as usize
}

pub unsafe fn sys_time(time_ptr: *mut Duration, realtime: bool) {
    syscall(SYS_TIME, time_ptr as u32, realtime as u32, 0);
}

pub unsafe fn sys_brk(addr: usize) -> usize {
    syscall(SYS_BRK, addr as u32, 0, 0) as usize
}

//TODO: Export unsafe
pub fn sys_yield() {
    unsafe {
        syscall(SYS_YIELD, 0, 0, 0);
    }
}

pub unsafe fn sys_trigger(event_ptr: *const Event) {
    syscall(SYS_TRIGGER, event_ptr as u32, 0, 0);
}

pub unsafe fn sys_window_create(ptr: *mut Window) {
    syscall(SYS_WINDOW_CREATE, ptr as u32, 0, 0);
}

pub unsafe fn sys_window_destroy(ptr: *mut Window) {
    syscall(SYS_WINDOW_DESTROY, ptr as u32, 0, 0);
}

pub unsafe fn sys_alloc(size: usize) -> usize {
    syscall(SYS_ALLOC, size as u32, 0, 0) as usize
}

pub unsafe fn sys_realloc(ptr: usize, size: usize) -> usize {
    syscall(SYS_REALLOC, ptr as u32, size as u32, 0) as usize
}

pub unsafe fn sys_realloc_inplace(ptr: usize, size: usize) -> usize {
    syscall(SYS_REALLOC_INPLACE, ptr as u32, size as u32, 0) as usize
}

pub unsafe fn sys_unalloc(ptr: usize) {
    syscall(SYS_UNALLOC, ptr as u32, 0, 0);
}
