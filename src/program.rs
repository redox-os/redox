#![crate_type="staticlib"]
#![allow(unused_features)]
#![feature(alloc)]
#![feature(asm)]
#![feature(box_syntax)]
#![feature(collections)]
#![feature(core_slice_ext)]
#![feature(no_std)]
#![feature(vec_push_all)]
#![feature(vec_resize)]
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
    let mut args: Vec<String> = Vec::new();
    for i in 0..argc as isize {
        let arg = ptr::read(stack.offset(1 + i)) as *const u8;
        if arg as usize > 0 {
            let mut utf8: Vec<u8> = Vec::new();
            for j in 0..4096 /* Max arg length */ {
                let b = ptr::read(arg.offset(j));
                if b == 0 {
                    break;
                }else{
                    utf8.push(b);
                }
            }
            args.push(String::from_utf8_unchecked(utf8));
        } else {
            args.push(String::new());
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
#[no_mangle]
pub unsafe fn _start() {
    let stack: *const u32;
    asm!("" : "={esp}"(stack) : : "memory" : "intel", "volatile");
    _start_stack(stack);
}
