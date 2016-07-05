#![crate_name="ralloc_shim"]
#![crate_type="lib"]
#![feature(lang_items)]
#![no_std]

extern crate system;

use system::syscall::unix::{sys_brk, sys_yield};

/// Cooperatively gives up a timeslice to the OS scheduler.
pub extern "C" fn sched_yield() -> isize {
    match sys_yield() {
        Ok(_) => 0,
        Err(_) => -1
    }
}

/// Increment data segment of this process by some, _n_, return a pointer to the new data segment
/// start.
///
/// This uses the system call BRK as backend.
///
/// This is unsafe for multiple reasons. Most importantly, it can create an inconsistent state,
/// because it is not atomic. Thus, it can be used to create Undefined Behavior.
pub extern "C" fn sbrk(n: isize) -> *mut u8 {
    let orig_seg_end = match unsafe { sys_brk(0) } {
        Ok(end) => end,
        Err(_) => return !0 as *mut u8
    };

    if n == 0 {
        return orig_seg_end as *mut u8;
    }

    let expected_end = match orig_seg_end.checked_add(n as usize) {
        Some(end) => end,
        None => return !0 as *mut u8
    };

    let new_seg_end = match unsafe { sys_brk(expected_end) } {
        Ok(end) => end,
        Err(_) => return !0 as *mut u8
    };

    if new_seg_end != expected_end {
        // Reset the break.
        let _ = unsafe { sys_brk(orig_seg_end) };

        !0 as *mut u8
    } else {
        orig_seg_end as *mut u8
    }
}
