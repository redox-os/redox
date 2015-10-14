use alloc::boxed::*;

use vec::Vec;

static mut _args: *mut Vec<&'static str> = 0 as *mut Vec<&'static str>;

/// Arguments
pub fn args<'a>() -> &'a Vec<&'static str> {
    unsafe { &*_args }
}

/// Initialize arguments
pub unsafe fn args_init(args: Vec<&'static str>) {
    _args = Box::into_raw(box args);
}

/// Destroy arguments
pub unsafe fn args_destroy() {
    if _args as usize > 0 {
        drop(Box::from_raw(_args));
    }
}
