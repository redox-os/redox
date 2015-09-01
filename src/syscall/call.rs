use alloc::boxed::*;

use common::event::*;
use common::net::*;
use common::resource::*;

use graphics::window::*;

use syscall::common::*;

pub unsafe fn syscall(eax: u32, ebx: u32, ecx: u32, edx: u32){
    asm!("int 0x80"
        :
        : "{eax}"(eax), "{ebx}"(ebx), "{ecx}"(ecx), "{edx}"(edx)
        : "memory"
        : "intel", "volatile");
}

pub fn sys_debug(byte: u8){
    unsafe{
        syscall(SYS_DEBUG, byte as u32, 0, 0);
    }
}

pub fn sys_exit() {
    unsafe{
        syscall(SYS_EXIT, 0, 0, 0);
    }
}

pub fn sys_open(url: &URL, resource_ptr: *mut Box<Resource>){
    unsafe{
        let url_ptr: *const URL = url;
        syscall(SYS_OPEN, url_ptr as u32, resource_ptr as u32, 0);
    }
}

pub fn sys_tcp_listener_create(ptr: *mut TcpListener){
    unsafe{
        syscall(SYS_TCP_LISTENER_CREATE, ptr as u32, 0, 0);
    }
}

pub fn sys_tcp_listener_destroy(ptr: *mut TcpListener){
    unsafe{
        syscall(SYS_TCP_LISTENER_DESTROY, ptr as u32, 0, 0);
    }
}

pub fn sys_trigger(event: &Event){
    unsafe{
        let event_ptr: *const Event = event;
        syscall(SYS_TRIGGER, event_ptr as u32, 0, 0);
    }
}

pub fn sys_window_create(ptr: *mut Window){
    unsafe{
        syscall(SYS_WINDOW_CREATE, ptr as u32, 0, 0);
    }
}

pub fn sys_window_destroy(ptr: *mut Window){
    unsafe{
        syscall(SYS_WINDOW_DESTROY, ptr as u32, 0, 0);
    }
}

pub fn sys_yield(){
    unsafe {
        syscall(SYS_YIELD, 0, 0, 0);
    }
}
