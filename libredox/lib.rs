#![crate_name="redox"]
#![crate_type="rlib"]
#![feature(alloc)]
#![feature(asm)]
#![feature(box_syntax)]
#![feature(core_simd)]
#![feature(core_slice_ext)]
#![feature(core_str_ext)]
#![feature(lang_items)]
#![feature(no_std)]
#![no_std]

extern crate alloc;

pub use alloc::boxed::*;

pub use audio::wav::*;

pub use common::event::*;
pub use common::queue::*;
pub use common::random::*;
pub use common::string::*;
pub use common::time::*;
pub use common::vec::*;

pub use graphics::bmp::*;
pub use graphics::color::*;
pub use graphics::consolewindow::*;
pub use graphics::display::*;
pub use graphics::point::*;
pub use graphics::size::*;
pub use graphics::window::*;

pub use externs::*;

pub use file::*;

pub use syscall::call::*;

#[path="../src/audio"]
mod audio{
    pub mod wav;
}

#[path="../src/common"]
mod common {
    pub mod debug; //Not needed
    pub mod event;
    pub mod queue;
    pub mod random; //Should remove
    pub mod scheduler; //Should remove
    pub mod string;
    pub mod time;
    pub mod vec;
}

#[path="../src/externs.rs"]
pub mod externs;

#[path="../src/graphics"]
mod graphics {
    pub mod bmp;
    pub mod color;
    pub mod consolewindow;
    pub mod display;
    pub mod point;
    pub mod size;
    pub mod window;
}

#[path="../src/syscall"]
mod syscall{
    pub mod call;
    pub mod common;
}

pub mod file;
