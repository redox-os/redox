#![crate_type="rlib"]
#![feature(associated_consts)]
#![feature(box_syntax)]
#![feature(no_std)]
#![no_std]

#[macro_use]
extern crate redox;

pub use bmp::BmpFile;
pub use color::Color;
pub use event::*;
pub use point::Point;
pub use size::Size;
pub use window::Window;

pub mod bmp;
pub mod color;
pub mod event;
pub mod point;
pub mod size;
pub mod window;
