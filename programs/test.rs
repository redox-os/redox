#![feature(asm)]
#![feature(core)]
#![feature(no_std)]
#![no_std]

extern crate core;

use core::str::StrExt;

pub unsafe fn outb(port: u16, value: u8){
    asm!("out $1, $0\n"
        : : "{al}"(value), "{dx}"(port) : : "intel");
}

const TEST: &'static str = "Test string from user application!\n";

#[no_mangle]
pub fn main() {
    for character in TEST.chars() {
        unsafe {
            outb(0x3F8, character as u8);
        }
    }
}