#![crate_name="system"]
#![crate_type="lib"]
#![feature(asm)]
#![feature(lang_items)]
#![no_std]

pub mod error;
pub mod externs;
pub mod scheme;
pub mod syscall;
