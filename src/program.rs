#![crate_type="staticlib"]
#![feature(asm)]
#![feature(box_syntax)]
#![feature(no_std)]
#![no_std]

#[macro_use]
extern crate redox;

use application::main;

use core::ptr;

#[path="APPLICATION_PATH"]
mod application;

use redox::*;

#[inline(never)]
unsafe fn _start_stack(stack: *const u32){
    let argc = ptr::read(stack);
    let mut args: Vec<String> = Vec::new();
    for i in 0..argc as isize {
        let arg = ptr::read(stack.offset(1 + i)) as *const u8;
        if arg as usize > 0 {
            args.push(String::from_c_str(arg));
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
pub unsafe fn _start(){
    let stack: *const u32;
    asm!("" : "={esp}"(stack) : : "memory" : "intel", "volatile");
    _start_stack(stack);
}
