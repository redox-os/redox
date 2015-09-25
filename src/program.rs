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

#[no_mangle]
pub unsafe fn _start(){
    ::redox::console::console_init();
    main();
    ::redox::console::console_destroy();
}
