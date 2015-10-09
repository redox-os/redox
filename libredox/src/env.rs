use core::mem::size_of;
use core::ptr;

use vec::Vec;

use syscall::{sys_alloc, sys_unalloc};

static mut _args: *mut Vec<&'static str> = 0 as *mut Vec<&'static str>;

/// Arguments
pub fn args<'a>() -> &'a Vec<&'static str> {
    unsafe { &*_args }
}

/// Initialize arguments
pub unsafe fn args_init(args: Vec<&'static str>) {
    _args = sys_alloc(size_of::<Vec<&'static str>>()) as *mut Vec<&'static str>;
    ptr::write(_args, args);
}

/// Destroy arguments
pub unsafe fn args_destroy() {
    drop(ptr::read(_args));
    sys_unalloc(_args as usize);
    _args = 0 as *mut Vec<&'static str>;
}
