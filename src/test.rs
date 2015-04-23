#![feature(asm)]
#![feature(core)]
#![feature(no_std)]
#![no_std]

extern crate core;

use common::debug::*;

mod common {
    pub mod debug;
	pub mod pio;
}

const TEST: &'static str = "Test string from user application!\n";

#[no_mangle]
pub fn main() {
    d(TEST);
}