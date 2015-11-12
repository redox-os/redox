#![crate_type="staticlib"]
#![allow(unused_features)]
#![feature(asm)]
#![feature(box_syntax)]
#![feature(iter_arith)]
#![feature(no_std)]
#![feature(slice_concat_ext)]
#![feature(vec_push_all)]
#![feature(vec_resize)]
#![no_std]

#[macro_use]
extern crate redox;

extern crate orbital;

use application::main;

#[path="APPLICATION_PATH"]
pub mod application;

use redox::*;
use redox::syscall::sys_exit;

#[no_mangle]
#[inline(never)]
pub unsafe extern fn _start_stack(stack: *const usize) {
    let mut args: Vec<&'static str> = Vec::new();
    //TODO: Fix issue with stack not being in context VM space
    let argc = ptr::read(stack);
    for i in 0..argc as isize {
        let arg = ptr::read(stack.offset(1 + i)) as *const u8;
        if arg as usize > 0 {
            let mut len = 0;
            for j in 0..4096 /* Max arg length */ {
                len = j;
                if ptr::read(arg.offset(j)) == 0 {
                    break;
                }
            }
            let utf8: &'static [u8] = slice::from_raw_parts(arg, len as usize);
            args.push(str::from_utf8_unchecked(utf8));
        }
    }

    args_init(args);
    main();
    args_destroy();
    sys_exit(0);
}

/*
#[cold]
#[inline(never)]
#[naked]
#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe fn _start() {
    let stack: *const usize;
    asm!("" : "={esp}"(stack) : : "memory" : "intel", "volatile");
    _start_stack(stack);
}
*/
