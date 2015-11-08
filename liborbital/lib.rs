#![crate_type="rlib"]
#![feature(associated_consts)]
#![feature(box_syntax)]
#![feature(no_std)]
#![no_std]

#[macro_use]
extern crate redox;

pub use bmp::BmpFile;
pub use event::*;
pub use graphics::color::Color;
pub use window::Window;

pub mod bmp;
pub mod console;
pub mod event;
pub mod graphics;
pub mod window;
