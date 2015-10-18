#![crate_type="staticlib"]
#![allow(unused_features)]
#![feature(alloc)]
#![feature(asm)]
#![feature(box_syntax)]
#![feature(collections)]
#![feature(convert)]
#![feature(core_slice_ext)]
#![feature(no_std)]
#![feature(vec_push_all)]
#![feature(vec_resize)]
#![feature(deque_extras)]
#![no_std]

#[macro_use]
extern crate alloc;

#[macro_use]
extern crate collections;

#[macro_use]
extern crate redox;

use application::main;

#[path="APPLICATION_PATH"]
mod application;

use redox::*;

use redox::syscall::sys_exit;

#[inline(never)]
unsafe fn _start_stack(stack: *const u32) {
    let argc = ptr::read(stack);
    let mut args: Vec<&'static str> = Vec::new();
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
    console_init();
    main();
    console_destroy();
    args_destroy();
    sys_exit(0);
}

#[cold]
#[inline(never)]
#[no_mangle]
pub unsafe extern "C" fn _start() {
    let stack: *const u32;
    asm!("" : "={esp}"(stack) : : "memory" : "intel", "volatile");
    _start_stack(stack);
}
