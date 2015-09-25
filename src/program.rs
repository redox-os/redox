#![crate_type="staticlib"]
#![feature(asm)]
#![feature(box_syntax)]
#![feature(no_std)]
#![no_std]

#[macro_use]
extern crate redox;

use application::main;

#[path="APPLICATION_PATH"]
mod application;

use redox::*;

#[no_mangle]
pub unsafe fn _start(){
    console_init();
    main();
    console_destroy();
    sys_exit(0);
}
