#![crate_type="lib"]
#![feature(associated_consts)]
#![feature(box_syntax)]
#![feature(deprecated)]

#![deny(warnings)]

extern crate core;

pub use bmp::BmpFile;
pub use color::Color;
pub use event::*;
pub use point::Point;
pub use size::Size;
pub use window::Window;

pub mod bmp;
pub mod color;
#[path="../kernel/common/event.rs"]
pub mod event;
pub mod point;
pub mod size;
pub mod window;
